use super::error::BBCError;
use super::BBCUrl;
use crate::common::{LinkTo, Page, ScrapableContent, UrlTrait};
use crate::Result;

use futures::stream::{self, StreamExt};
use itertools::Itertools;
use scraper::{ElementRef, Html};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, rc::Rc};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct BBCContent {
    title: String,
    content: Vec<String>,
    metadata: Metadata,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct Metadata {
    related_topics: Vec<String>,
    timestamp: String,
    page_links: HashSet<Page<LinkTo, BBCUrl>>,
}
impl ScrapableContent for BBCContent {
    type Url = BBCUrl;
    fn from_scraped_page(url: &Self::Url, document: &Html) -> Result<Self> {
        // TODO Break this function into smaller functions to be able to run async
        println!("Scraping article: {:?}", url);

        let article = Self::extract_article(&document).ok_or(BBCError::NoArticleFound {
            url: url.full_url(),
        })?;

        let title = Self::extract_title(&article).ok_or(BBCError::NoTitleFound {
            url: url.full_url(),
        })?;

        let content = Self::extract_content(&article).ok_or(BBCError::NoContentFound {
            url: url.full_url(),
        })?;

        // Image Selector; to Vec<Image(Url, Caption)>

        let related_topics =
            Self::extract_related_topics(&article).ok_or(BBCError::NoRelatedTopicsFound {
                url: url.full_url(),
            })?;

        let timestamp = Self::extract_timestamp(&article);

        let page_links = Self::extract_related_links(&article)
            .into_iter()
            .filter_map(|(url, title)| {
                let url = BBCUrl::try_from(url).ok()?;
                let title = title.clone();
                Some(Page::<LinkTo, BBCUrl>::new(url, title))
            })
            .collect::<HashSet<Page<LinkTo, BBCUrl>>>();

        Ok(BBCContent::new(
            title,
            content,
            related_topics,
            timestamp,
            page_links,
        ))
    }

    fn get_related_pages(&self) -> HashSet<Page<LinkTo, Self::Url>> {
        self.metadata.page_links.clone()

    }
}

impl BBCContent {
    fn new(
        title: String,
        content: Vec<String>,
        related_topics: Vec<String>,
        timestamp: String,
        page_links: HashSet<Page<LinkTo, BBCUrl>>,
    ) -> Self {
        BBCContent {
            title,
            content,
            metadata: Metadata {
                related_topics,
                timestamp,
                page_links,
            },
        }
    }

    fn extract_article<'a>(document: &'a Html) -> Option<ElementRef<'a>> {
        let article_selector = scraper::Selector::parse("article").unwrap();
        document.select(&article_selector).next()
    }

    fn extract_title<'a>(article: &'a ElementRef) -> Option<String> {
        let title_selector = scraper::Selector::parse("h1").unwrap();
        article
            .select(&title_selector)
            .next()
            .map(|title| title.text().collect::<String>())
    }
    fn extract_content<'a>(article: &'a ElementRef) -> Option<Vec<String>> {
        let content_selector =
            scraper::Selector::parse("div[data-component='text-block']").unwrap();
        let content = article
            .select(&content_selector)
            .map(|element| element.text().collect::<String>())
            .collect::<Vec<String>>();

        if content.is_empty() {
            None
        } else {
            Some(content)
        }
    }
    fn extract_related_topics<'a>(article: &'a ElementRef) -> Option<Vec<String>> {
        let related_topics_selector =
            scraper::Selector::parse("div[data-component='topic-list']").unwrap();
        let related_topics = article.select(&related_topics_selector).next()?;
        let related_topics_selector_name = scraper::Selector::parse("li").unwrap();
        // Can have empty related topics
        Some(
            related_topics
                .select(&related_topics_selector_name)
                .map(|element| element.text().collect::<String>())
                .collect::<Vec<String>>(),
        )
    }
    fn extract_related_links(article: &ElementRef) -> Vec<(String, String)> {
        let related_links_selector = scraper::Selector::parse("a").unwrap();
        article
            .select(&related_links_selector)
            .filter_map(|element| {
                let url = element.value().attr("href")?.to_string();
                let text = element.text().collect::<String>();
                Some((url, text))
            })
            .collect::<Vec<(String, String)>>()
    }
    fn extract_timestamp(article: &ElementRef) -> String {
        // <time data-testid="timestamp" datetime="2024-06-10T06:58:21.378Z">10 June 2024, 07:58 BST</time>
        // I want to extract out the datetime attribute
        let timestamp_selector = scraper::Selector::parse("time").unwrap();
        article
            .select(&timestamp_selector)
            .next()
            .and_then(|element| element.value().attr("datetime"))
            .unwrap_or_default()
            .to_string()
    }
}
