use std::collections::HashSet;

use crate::scraper::error::{Error, Result};
use crate::scraper::make_request::make_request;
use crate::scraper::url::{BBCUrl, Url};
use scraper::{ElementRef, Html};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt::Display};

use std::hash::{Hash, Hasher};
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct URL(pub String);

impl URL {
    pub fn new(url: impl Into<String>) -> Option<Self> {
        Self::parse_url(url.into()).map(Self)
    }

    pub fn url(&self) -> String {
        self.0.clone()
    }

    fn parse_url(url: String) -> Option<String> {
        let base_url = "https://www.bbc.co.uk";
        let stripped_url = url.strip_prefix(base_url).unwrap_or(&url);
        if !stripped_url.starts_with("/news") {
            return None;
        }

        let segments: Vec<&str> = stripped_url.split("/").collect();
        let ends_with_number = segments.last().map_or(false, |last_segment| {
            last_segment.split('-').last().map_or(false, |last_word| {
                last_word.len() == 8 && last_word.parse::<u32>().is_ok()
            })
        });
        let has_article_or_number = segments
            .get(2)
            .map_or(false, |&segment| segment == "article" || ends_with_number);
        let no_live_segment = !segments.iter().any(|&segment| segment == "live");

        if has_article_or_number && no_live_segment {
            Some(format!("{}{}", base_url, stripped_url).to_string())
        } else {
            None
        }
    }

    fn from_list(urls: Vec<String>) -> Vec<Self> {
        urls.into_iter().filter_map(Self::new).collect()
    }

    fn to_article(self) -> Article {
        Article::new(self, "", "", Vec::<String>::new())
    }
}
impl Hash for URL {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl Display for URL {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BBCNewsArticle {
    title: String,

    content: Vec<String>,
    related_topics: Vec<String>,
    timestamp: Vec<String>,
    //images: Vec<Image>,
}

impl Article {
    pub fn new(
        url: URL,
        title: impl Into<String>,
        content: impl Into<String>,
        related_topics: Vec<impl Into<String>>,
        //images: Vec<Image>,
    ) -> Self {
        Self {
            url,
            title: title.into().into(),
            content: content.into().into(),
            related_topics: related_topics
                .into_iter()
                .map(|topic| topic.into().into())
                .collect(),
            //images,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkedArticle {
    pub article: Article,
    pub links: Vec<URL>,
}

impl LinkedArticle {
    pub fn new(article: Article, links: Vec<String>) -> Self {
        Self {
            article,
            links: links
                .into_iter()
                .filter_map(|link| URL::new(link))
                .collect(),
        }
    }
}

const BASE_URL: &str = "https://www.bbc.co.uk";

pub async fn scrape_base_url() -> Result<HashSet<URL>> {
    println!("Scraping base url!");
    let base_url = URL("https://www.bbc.co.uk/news".to_string());
    let base_url = Url::BBC(BBCUrl(base_url.url()));
    let mut urls = Vec::new();

    let document = make_request(&base_url).await?;
    urls.extend(scrape_headline(&document)?);
    urls.extend(scrape_most_read(&document)?);
    let urls = URL::from_list(urls);
    // Turn vec to hash-set
    let urls: HashSet<URL> = urls.into_iter().collect();
    Ok(urls)
}

fn scrape_most_read(document: &Html) -> Result<Vec<String>> {
    println!("Scraping most read!");
    let most_read_selector = scraper::Selector::parse("div[data-component='mostRead']").unwrap();
    let most_read = document.select(&most_read_selector).next().unwrap();
    let link_selector = scraper::Selector::parse("a").unwrap();
    let mut urls = Vec::new();

    for element in most_read.select(&link_selector) {
        let text = element.text().collect::<String>();
        let link = element.value().attr("href").unwrap();
        println!("{:?} :: {:?}", text, link);
        urls.push(link.to_string());
    }

    if urls.is_empty() {
        Err(Error::NoUrlsFound)
    } else {
        Ok(urls)
    }
}

fn scrape_headline(document: &Html) -> Result<Vec<String>> {
    println!("Scraping headline!");
    let headline_selector = scraper::Selector::parse("div#nations-news-uk").unwrap();
    let headline = document.select(&headline_selector).next().unwrap();
    let link_selector = scraper::Selector::parse("a").unwrap();
    let mut urls = Vec::new();

    for element in headline.select(&link_selector) {
        let text = element.text().collect::<String>();
        let link = element.value().attr("href").unwrap();
        println!("{:?} :: {:?}", text, link);
        urls.push(link.to_string());
    }

    if urls.is_empty() {
        Err(Error::NoUrlsFound)
    } else {
        Ok(urls)
    }
}

fn extract_article<'a>(document: &'a Html) -> Result<ElementRef<'a>> {
    let article_selector = scraper::Selector::parse("article").unwrap();
    document
        .select(&article_selector)
        .next()
        .ok_or(Error::NoArticleFound)
}

fn extract_title<'a>(article: &'a ElementRef) -> Result<String> {
    let title_selector = scraper::Selector::parse("h1").unwrap();
    let title = article
        .select(&title_selector)
        .next()
        .ok_or(Error::NoTitleFound)?;
    Ok(title.text().collect::<String>())
}

fn extract_content<'a>(article: &'a ElementRef) -> Result<String> {
    let content_selector = scraper::Selector::parse("div[data-component='text-block']").unwrap();
    let content = article
        .select(&content_selector)
        .map(|element| element.text().collect::<String>())
        .collect::<Vec<String>>()
        .join("\n\n");
    if content.is_empty() {
        Err(Error::NoContentFound)
    } else {
        Ok(content)
    }
}

fn extract_related_topics<'a>(article: &'a ElementRef) -> Result<Vec<String>> {
    let related_topics_selector =
        scraper::Selector::parse("div[data-component='topic-list']").unwrap();
    let related_topics = article
        .select(&related_topics_selector)
        .next()
        .ok_or(Error::NoRelatedTopicsFound)?;
    let related_topics_selector_name = scraper::Selector::parse("li").unwrap();
    // Can have empty related topics
    Ok(related_topics
        .select(&related_topics_selector_name)
        .map(|element| element.text().collect::<String>())
        .collect::<Vec<String>>())
}

fn extract_related_links(article: &ElementRef) -> Vec<String> {
    let related_links_selector = scraper::Selector::parse("a").unwrap();
    article
        .select(&related_links_selector)
        .filter_map(|element| element.value().attr("href"))
        .map(|href| href.to_string())
        .collect::<Vec<String>>()
}

// TODO Break this function into smaller functions to be able to run async
fn scrape_article(url: &URL, document: &Html) -> Result<LinkedArticle> {
    println!("Scraping article: {:?}", url);

    let article = extract_article(&document)?;

    let title = extract_title(&article)?;

    let content = extract_content(&article)?;

    // Image Selector; to Vec<Image(Url, Caption)>

    let related_topics = extract_related_topics(&article)?;

    let related_links = extract_related_links(&article);

    let article = Article::new(url.clone(), title, content, related_topics);

    let linked_article = LinkedArticle::new(article, related_links);
    Ok(linked_article)
}

pub async fn scrape_page(url: &URL) -> Result<LinkedArticle> {
    let request_url = Url::BBC(BBCUrl(url.url()));
    let document = make_request(&request_url).await?;
    Ok(scrape_article(url, &document)?)
}
