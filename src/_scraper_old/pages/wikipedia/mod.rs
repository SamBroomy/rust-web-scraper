#![allow(unused)] // For beginning only.
use std::result::Result as StdResult;
mod error;
use crate::scraper::make_request::make_request;
use crate::scraper::url::{Url, UrlTrait, WikipediaUrl};
use error::{Error, Result};
use futures::stream::futures_unordered::IntoIter;
use itertools::Itertools;
use lazy_regex::{lazy_regex, regex, regex_is_match, regex_replace_all};
use lazy_static::lazy_static;
use scraper::{ElementRef, Html, Selector};
use serde::{de::Error as DeError, de::SeqAccess, de::Visitor, Deserialize, Deserializer};
use serde::{Serialize, Serializer};
use serde_json::json;
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};
use std::fmt::format;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tokio::task::LocalEnterGuard;

fn parse_text(text: String) -> Option<String> {
    // Remove all text within square brackets like references and citations
    let parsed = regex_replace_all!(r"\[.*?\]", text.as_ref(), "");
    // Remove all that are unicode escape sequences
    let parsed = regex_replace_all!(r"\\u\{\w{1,6}\}", parsed.as_ref(), " ");
    // If string starts or ends with a . empty it
    let parsed = regex_replace_all!(r"^\..*\.$", parsed.as_ref(), "");
    let parsed = parsed.trim();
    if parsed.is_empty() {
        None
    } else {
        Some(parsed.to_string())
    }
}

// Function to determine if a table is complex based on the presence of rowspan or colspan
fn is_complex_table(table: &ElementRef) -> bool {
    let cell_sel = Selector::parse("th, td").unwrap();
    table.select(&cell_sel).any(|cell| {
        cell.value().attr("rowspan").is_some() || cell.value().attr("colspan").is_some()
    })
}

fn extract_simple_table_data(table: &ElementRef) -> Option<Table> {
    let row_sel = Selector::parse("tr").unwrap();
    let header_sel = Selector::parse("th").unwrap();
    let cell_sel = Selector::parse("td").unwrap();

    let headers: Vec<String> = table.select(&row_sel).next().map_or(vec![], |r| {
        r.select(&header_sel)
            .map(|h| h.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .collect()
    });

    if headers.is_empty() {
        return None; // Skip tables without headers
    }

    // Initialize columns with titles
    let mut columns: Vec<Column> = headers
        .into_iter()
        .map(|title| Column {
            title,
            data: vec![],
        })
        .collect();

    // Fill column data from table cells
    for row in table.select(&row_sel).skip(1) {
        // Skip header row
        row.select(&cell_sel).enumerate().for_each(|(i, cell)| {
            if let Some(column) = columns.get_mut(i) {
                column
                    .data
                    .push(cell.text().collect::<Vec<_>>().join(" ").trim().to_string());
            }
        });
    }
    Table::new(columns)
}
// Adjusted function to work with the Table struct
fn handle_rowspan_and_colspan(table: ElementRef) -> Table {
    let row_sel = Selector::parse("tr").unwrap();
    let header_sel = Selector::parse("th").unwrap();
    let cell_sel = Selector::parse("td").unwrap();

    // Initialize an empty table
    let mut table_struct = Table {
        columns: Vec::new(),
    };
    let mut rowspan_states: HashMap<usize, (usize, String)> = HashMap::new();

    for (row_index, row) in table.select(&row_sel).enumerate() {
        let mut col_index = 0;

        if row_index == 0 {
            // Assuming the first row is headers
            for header in row.select(&header_sel) {
                let title = header
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
                    .to_string();
                table_struct.columns.push(Column {
                    title,
                    data: Vec::new(),
                });
            }
        } else {
            for cell in row.select(&cell_sel) {
                // Skip columns affected by previous rows' rowspan
                while rowspan_states.contains_key(&col_index) {
                    if let Some((rows_left, content)) = rowspan_states.get_mut(&col_index) {
                        if *rows_left > 1 {
                            *rows_left -= 1;
                            table_struct.columns[col_index].data.push(content.clone());
                        } else {
                            rowspan_states.remove(&col_index);
                        }
                    }
                    col_index += 1;
                }

                let cell_text = cell.text().collect::<Vec<_>>().join(" ").trim().to_string();
                let rowspan = cell
                    .value()
                    .attr("rowspan")
                    .and_then(|r| r.parse::<usize>().ok())
                    .unwrap_or(1);
                let colspan = cell
                    .value()
                    .attr("colspan")
                    .and_then(|c| c.parse::<usize>().ok())
                    .unwrap_or(1);

                // Handle colspan and rowspan
                for _ in 0..colspan {
                    if col_index < table_struct.columns.len() {
                        table_struct.columns[col_index].data.push(cell_text.clone());
                        if rowspan > 1 {
                            rowspan_states.insert(col_index, (rowspan, cell_text.clone()));
                        }
                    }
                    col_index += 1;
                }
            }
        }

        // Fill the remaining columns for this row if affected by a previous rowspan but not the current row
        while col_index < table_struct.columns.len() {
            if let Some((rows_left, content)) = rowspan_states.get_mut(&col_index) {
                if *rows_left > 1 {
                    *rows_left -= 1;
                    table_struct.columns[col_index].data.push(content.clone());
                } else {
                    rowspan_states.remove(&col_index);
                }
            } else {
                table_struct.columns[col_index].data.push(String::new());
            }
            col_index += 1;
        }
    }

    table_struct
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Table {
    columns: Vec<Column>,
}

impl Table {
    pub fn new(columns: Vec<Column>) -> Option<Self> {
        // Remove any columns that are empty or have no data
        let columns: Vec<Column> = columns
            .into_iter()
            .filter_map(|column| Column::new(column.title, column.data))
            .filter(|column| !column.data.iter().all(|text| text.is_empty()))
            .collect();
        if columns.is_empty() {
            None
        } else {
            Some(Self { columns })
        }
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
struct Column {
    title: String,
    data: Vec<String>,
}

impl Column {
    pub fn new(title: String, data: Vec<String>) -> Option<Self> {
        let data: Vec<String> = data
            .into_iter()
            .map(|text| parse_text(text).unwrap_or_else(String::new))
            .collect();

        if data.iter().all(|text| text.is_empty()) {
            None
        } else {
            Some(Self {
                title: parse_text(title)?,
                data,
            })
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Link {
    title: String,
    url: WikipediaUrl,
}
impl Link {
    pub fn new(title: String, url: impl Into<String>) -> Option<Self> {
        Some(Self {
            title: parse_text(title)?,
            url: WikipediaUrl::new(url.into())?,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Figure {
    caption: String,
    alt_text: Option<String>,
    true_url: String,
    wiki_url: WikipediaUrl,
}
impl Figure {
    pub fn new(
        caption: String,
        alt_text: String,
        true_url: String,
        wiki_url: String,
    ) -> Option<Self> {
        Some(Self {
            caption: parse_text(caption)?,
            alt_text: parse_text(alt_text),
            true_url: true_url,
            wiki_url: WikipediaUrl::new(wiki_url)?,
        })
    }
}
#[derive(Debug, Clone)]
pub enum SectionContentType {
    Content(Vec<SectionContent>),
    Empty,
}
impl Serialize for SectionContentType {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            SectionContentType::Content(ref contents) => contents.serialize(serializer),
            SectionContentType::Empty => serializer.serialize_unit(),
        }
    }
}

impl<'de> Deserialize<'de> for SectionContentType {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SectionContentTypeVisitor;

        impl<'de> Visitor<'de> for SectionContentTypeVisitor {
            type Value = SectionContentType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a nonempty array for Content or null for Empty")
            }

            fn visit_seq<A>(self, mut seq: A) -> StdResult<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut contents = Vec::new();
                while let Some(content) = seq.next_element()? {
                    contents.push(content);
                }
                Ok(SectionContentType::Content(contents))
            }

            fn visit_unit<E>(self) -> StdResult<Self::Value, E>
            where
                E: DeError,
            {
                Ok(SectionContentType::Empty)
            }
        }

        deserializer.deserialize_any(SectionContentTypeVisitor)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
enum ContentValue {
    Text(String),
    Link(Link),
    Figure(Figure),
    Table(Table),
    Nested(Box<SectionContent>),
}

impl ContentValue {
    fn get_nested(&mut self) -> Option<&mut SectionContent> {
        if let ContentValue::Nested(content) = self {
            Some(content)
        } else {
            None
        }
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SectionContent {
    heading: String,
    level: u32,
    content: Vec<ContentValue>,
}

impl SectionContent {
    pub fn new(heading: String, level: u32) -> Option<Self> {
        Some(Self {
            heading: parse_text(heading)?,
            level,
            content: Vec::new(),
        })
    }

    fn is_empty_content(&self) -> bool {
        self.content.is_empty()
    }

    fn add_nested(&mut self, nested: SectionContent) {
        self.content.push(ContentValue::Nested(Box::new(nested)));
    }
    fn add_text(&mut self, text: String) {
        if let Some(text) = parse_text(text) {
            self.content.push(ContentValue::Text(text));
        }
    }
    fn add_link(&mut self, link: Link) {
        self.content.push(ContentValue::Link(link));
    }

    fn add_figure(&mut self, figure: Figure) {
        self.content.push(ContentValue::Figure(figure));
    }
    fn add_table(&mut self, table: Table) {
        self.content.push(ContentValue::Table(table));
    }

    // TODO rework function, break into smaller functions and output a result
    fn extract_content<'a, I>(mut content_iter: I) -> Result<Vec<SectionContent>>
    where
        I: Iterator<Item = ElementRef<'a>>,
    {
        let mut contents = Vec::<SectionContent>::new();
        let mut stack = VecDeque::<SectionContent>::new();

        while let Some(element) = content_iter.next() {
            let heading_name = element.value().name();

            match heading_name {
                name if regex_is_match!(r"h\d", name) => {
                    let new_section_level =
                        heading_name.chars().nth(1).unwrap().to_digit(10).unwrap();

                    while stack.back().map_or(false, |s| s.level >= new_section_level) {
                        if let Some(section) = stack.pop_back() {
                            // Todo extract into function
                            if !section.is_empty_content() {
                                if let Some(parent) = stack.back_mut() {
                                    parent.add_nested(section);
                                } else {
                                    contents.push(section);
                                }
                            }
                        }
                    }
                    if let Some(section) =
                        SectionContent::new(element.text().collect::<String>(), new_section_level)
                    {
                        stack.push_back(section);
                    }
                }

                "p" => {
                    if let Some(last_section) = stack.back_mut() {
                        last_section.add_text(element.text().collect::<String>());
                    }
                }

                "ul" => {
                    if let Some(last_section) = stack.back_mut() {
                        for list_item in element.select(&Selector::parse("li").unwrap()) {
                            last_section.add_text(list_item.text().collect::<String>());
                        }
                    }
                }
                // blockquote
                "div" => {
                    if let Some(last_section) = stack.back_mut() {
                        if element.attr("role") == Some("note") {
                            element
                                .select(&Selector::parse("a").unwrap())
                                .for_each(|link| {
                                    let title = link.text().collect::<String>();
                                    let url = link
                                        .value()
                                        .attr("href")
                                        .map_or("".to_string(), |href| href.to_string());
                                    if let Some(link) = Link::new(title, url) {
                                        last_section.add_link(link);
                                    }
                                });
                        }
                    }
                }
                "figure" => {
                    if let Some(last_section) = stack.back_mut() {
                        let img = element.select(&Selector::parse("img").unwrap()).next();
                        if let Some(img) = img {
                            let alt_text = img.value().attr("alt").unwrap_or("").to_string();
                            let true_url = img.value().attr("src").unwrap_or("").to_string();
                            let caption = element
                                .select(&Selector::parse("figcaption").unwrap())
                                .next()
                                .map_or("".to_string(), |caption| {
                                    caption.text().collect::<String>()
                                });
                            let wiki_url = element
                                .select(&Selector::parse("a").unwrap())
                                .next()
                                .map_or("".to_string(), |link| {
                                    link.value()
                                        .attr("href")
                                        .map_or("".to_string(), |href| href.to_string())
                                });

                            if let Some(figure) = Figure::new(caption, alt_text, true_url, wiki_url)
                            {
                                last_section.add_figure(figure);
                            }
                        }
                    }
                }
                "table" => {
                    if let Some(last_section) = stack.back_mut() {
                        let table = extract_simple_table_data(&element);

                        if let Some(table) = table {
                            last_section.add_table(table);
                        }
                    }
                }
                "dl" => {
                    if let Some(last_section) = stack.back_mut() {
                        let title = element
                            .select(&Selector::parse("dt").unwrap())
                            .next()
                            .map_or("".to_string(), |title| title.text().collect::<String>());
                        if let Some(mut section) =
                            SectionContent::new(title, last_section.level + 1)
                        {
                            for element in element.select(&Selector::parse("dd").unwrap()) {
                                section.add_text(element.text().collect::<String>());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        while let Some(section) = stack.pop_back() {
            if !section.is_empty_content() {
                if let Some(parent) = stack.back_mut() {
                    parent.add_nested(section);
                } else {
                    contents.push(section);
                }
            } else {
                println!("Empty Section: {:?}", section);
            }
        }

        Ok(contents)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WikipediaPage {
    title: String,
    short_description: String,
    //table: Table,
    abstract_text: Vec<String>,
    content: SectionContentType,
    categories: Vec<Link>,
    page_links: HashSet<WikipediaUrl>,
}

impl WikipediaPage {
    pub fn new(
        title: String,
        short_description: String,
        //table: Table,
        abstract_text: Vec<String>,
        content: SectionContentType,
        categories: Vec<Link>,
        page_links: HashSet<WikipediaUrl>,
    ) -> Option<Self> {
        Some(Self {
            title: parse_text(title)?,
            short_description: parse_text(short_description)?,
            //table,
            abstract_text: abstract_text
                .into_iter()
                .filter_map(|text| parse_text(text))
                .collect(),
            content,
            categories,
            page_links,
        })
    }

    fn scrape_main_content<'a>(document: &'a Html) -> Result<ElementRef<'a>> {
        let main_selector = Selector::parse("main").unwrap();
        let main = document
            .select(&main_selector)
            .next()
            .ok_or(Error::NoPageContentFound)?;
        Ok(main)
    }

    fn scrape_body_content<'a>(document: &'a ElementRef<'a>) -> Result<ElementRef<'a>> {
        let body_selector = Selector::parse("div.mw-content-ltr").unwrap();
        let body = document
            .select(&body_selector)
            .next()
            .ok_or(Error::NoPageContentFound)?;
        Ok(body)
    }

    fn scrape_title(document: &ElementRef) -> Result<String> {
        let title_selector = Selector::parse("h1#firstHeading").unwrap();
        let title = document
            .select(&title_selector)
            .next()
            .ok_or(Error::NoTitleFound)?
            .text()
            .collect::<String>();
        Ok(title)
    }

    async fn get_page(url: &WikipediaUrl, extract_content: bool) -> Result<WikipediaPage> {
        let url: Url = url.clone().into();
        let page = make_request(&url).await?;

        let main = Self::scrape_main_content(&page)?;

        let title = Self::scrape_title(&main)?;

        let body_content = Self::scrape_body_content(&main)?;

        // Create an iterator over the child elements of the body content so we can peek ahead and pull out the first elements up to the first h2.
        let mut iter = body_content.child_elements().peekable();

        // Use peeking_take_while to take elements until we hit an h2, which we do not consume
        let abstract_iter = iter
            .by_ref()
            .peeking_take_while(|x| x.value().name() != "h2");
        let (short_description, table, abstract_text) =
            Self::extract_from_abstract_iter(abstract_iter)?;
        // At this point, iter will continue from the first h2 element
        // Collect the rest of the content elements for further processing
        // TODO - Extract table info
        // Error: Unable to extract content
        let content = if extract_content {
            SectionContentType::Content(SectionContent::extract_content(iter)?)
        } else {
            SectionContentType::Empty
        };

        let categories = main
            .select(&Selector::parse("div#catlinks").unwrap())
            .next()
            .ok_or(Error::NoCategoriesFound)?
            .select(&Selector::parse("ul").unwrap())
            .next()
            .ok_or(Error::NoCategoriesFound)?
            .select(&Selector::parse("a").unwrap())
            .into_iter()
            .filter_map(|category| {
                let title = category.text().collect::<String>();
                let url = category
                    .value()
                    .attr("href")
                    .map_or("".to_string(), |href| href.to_string());
                Link::new(title, url)
            })
            .collect::<Vec<Link>>();

        let page_links = main
            .select(&Selector::parse("a").unwrap())
            .into_iter()
            .filter_map(|link| {
                let url = link
                    .value()
                    .attr("href")
                    .map_or("".to_string(), |href| href.to_string());
                WikipediaUrl::new(url)
            })
            .collect::<HashSet<WikipediaUrl>>();

        WikipediaPage::new(
            title,
            short_description,
            //table,
            abstract_text,
            content,
            categories,
            page_links,
        )
        .ok_or(Error::UnableToExtractContent)
    }

    fn extract_from_abstract_iter<'a, I>(
        mut abstract_iter: I,
    ) -> Result<(String, Option<ElementRef<'a>>, Vec<String>)>
    where
        I: Iterator<Item = ElementRef<'a>>,
    {
        let mut short_description = String::new();
        let mut table_element = None;
        let mut abstract_text = Vec::new();

        while let Some(element) = abstract_iter.next() {
            match element.value().name() {
                "div"
                    if element
                        .value()
                        .classes()
                        .any(|class| class == "shortdescription") =>
                {
                    short_description = element.text().collect::<String>();
                }
                "table"
                    if element
                        .attr("class")
                        .map_or(false, |class| class.contains("infobox")) =>
                {
                    table_element = Some(element);
                }

                _ => {
                    if element.value().name() == "p" {
                        abstract_text.push(element.text().collect::<String>());
                    }
                }
            }
        }

        Ok((short_description, table_element, abstract_text))
    }

    pub async fn get_page_with_content(url: &WikipediaUrl) -> Result<WikipediaPage> {
        Self::get_page(url, true).await
    }

    pub async fn get_page_without_content(url: &WikipediaUrl) -> Result<WikipediaPage> {
        Self::get_page(url, false).await
    }

    fn get_category_links(&self) -> HashSet<WikipediaUrl> {
        self.categories
            .iter()
            .map(|link| link.url.clone())
            .collect::<HashSet<WikipediaUrl>>()
    }
    fn extract_urls_from_content(content: &SectionContent) -> HashSet<WikipediaUrl> {
        let mut urls = HashSet::new();
        for value in content.content.iter() {
            match value {
                ContentValue::Link(link) => {
                    urls.insert(link.url.clone());
                }
                ContentValue::Nested(nested) => {
                    urls.extend(Self::extract_urls_from_content(nested));
                }
                _ => {}
            }
        }
        urls
    }

    pub fn get_all_page_links(&self) -> HashSet<WikipediaUrl> {
        let mut urls = self.page_links.clone();
        match &self.content {
            SectionContentType::Content(content) => {
                for section in content {
                    urls.extend(Self::extract_urls_from_content(section));
                }
            }
            _ => {}
        }

        urls
    }
}
