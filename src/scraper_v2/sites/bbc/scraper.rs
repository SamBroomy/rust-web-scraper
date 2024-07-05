use crate::Result;

use super::{error::BBCError, BBCContent, BBCUrl};
use crate::common::{make_request, LinkTo, Page, Scrapable, SiteSpecificScraper, UrlTrait};

use async_trait::async_trait;
use scraper::Html;
use std::collections::HashSet;

pub struct BBCScraper;

#[async_trait]
impl SiteSpecificScraper for BBCScraper {
    type Url = BBCUrl;

    async fn scrape_initial_urls(&self) -> Result<Vec<Box<Page<dyn Scrapable, Self::Url>>>> {
        let initial_url = BBCUrl::initialise();
        let mut urls = Vec::new();

        let document = make_request(&initial_url).await?;

        let headline = Self::scrape_headline(&document);

        let most_read = Self::scrape_most_read(&document);

        urls.extend(headline);
        urls.extend(most_read);

        if urls.is_empty() {
            Err(BBCError::InitializeError {
                url: initial_url.full_url(),
            }
            .into())
        } else {
            let urls = urls
                .into_iter()
                .map(|page| Box::new(page) as Box<Page<dyn Scrapable, Self::Url>>)
                .collect::<Vec<Box<Page<dyn Scrapable, Self::Url>>>>();
            Ok(urls)
        }
    }
}

impl BBCScraper {
    fn scrape_section(document: &Html, selector_str: &str) -> HashSet<Page<LinkTo, BBCUrl>> {
        let selector = scraper::Selector::parse(selector_str).unwrap();
        let section = document.select(&selector).next().unwrap();

        let page_links = BBCContent::extract_related_links(&section);
        BBCContent::convert_to_page(page_links)
    }

    fn scrape_headline(document: &Html) -> HashSet<Page<LinkTo, BBCUrl>> {
        Self::scrape_section(document, "div#nations-news-uk")
    }

    fn scrape_most_read(document: &Html) -> HashSet<Page<LinkTo, BBCUrl>> {
        Self::scrape_section(document, "div[data-component='mostRead']")
    }
}
