pub mod bbc;
mod wikipedia;

use std::collections::HashSet;

//use bbc::BBCNewsPage;
use crate::scraper::url::Url;
pub use wikipedia::WikipediaPage;

#[derive(Clone)]
pub enum ScrapedPage {
    BBCNews(BBCNewsPage),
    //BBCSport(BBCSportPage),
    Wikipedia(WikipediaPage),
}

impl ScrapedPage {
    pub fn get_urls(&self) -> HashSet<Url> {
        match self {
            ScrapedPage::Wikipedia(page) => page
                .get_all_page_links()
                .into_iter()
                .map(Url::Wikipedia)
                .collect(),
            ScrapedPage::BBCNews(page) => page
                .get_all_page_links()
                .into_iter()
                .map(Url::BBC)
                .collect(),
        }
    }
}
