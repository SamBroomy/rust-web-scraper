use super::error::WikipediaError;
use super::WikipediaUrl;
use crate::common::{LinkTo, Page, ScrapableContent, UrlTrait};
use crate::Result;

use scraper::{ElementRef, Html};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, rc::Rc};

#[derive(Debug, Clone, Deserialize, Serialize)]
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
    fn from_scraped_page(url: &Self::Url, document: &Html) -> Result<Self> {
        // Parse the HTML to create a BBCPage
        todo!()
    }
}
