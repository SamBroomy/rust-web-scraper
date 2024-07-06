use super::WikipediaUrl;
use crate::common::{LinkTo, Page, RelatedPage, ScrapableContent};
use crate::Result;

use scraper::Html;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct WikipediaContent {
    title: String,
    short_description: String,
    //table: Table,
    abstract_text: Vec<String>,
    //content: SectionContentType,
    //categories: Vec<Link>,
    page_links: HashSet<WikipediaUrl>,
}
impl ScrapableContent for WikipediaContent {
    type Url = WikipediaUrl;
    type RelatedUrl = WikipediaUrl;
    fn from_scraped_page(url: &Self::Url, document: &Html) -> Result<Self> {
        // Parse the HTML to create a BBCPage
        todo!()
    }

    fn get_related_pages(&self) -> HashSet<Page<LinkTo, Self::Url>> {
        todo!()
    }

    fn get_related_topics(&self) -> HashSet<RelatedPage<Self::RelatedUrl>> {
        todo!()
    }

    fn get_title(&self) -> String {
        todo!()
    }
    fn get_url(&self) -> Self::Url {
        todo!()
    }
}
