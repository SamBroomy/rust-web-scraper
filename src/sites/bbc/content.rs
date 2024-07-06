use super::error::BBCError;
use super::{BBCNewsUrl, BBCRelatedTopicUrl};
use crate::common::{LinkTo, Page, RelatedPage, ScrapableContent, UrlTrait};
use crate::Result;

use scraper::{ElementRef, Html};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct BBCContent {
    title: String,
    content: Vec<String>,
    metadata: Metadata,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct Metadata {
    url: BBCNewsUrl,
    related_topics: HashSet<RelatedPage<BBCRelatedTopicUrl>>,
    timestamp: String,
    page_links: HashSet<Page<LinkTo, BBCNewsUrl>>,
}

impl ScrapableContent for BBCContent {
    type Url = BBCNewsUrl;
    type RelatedUrl = BBCRelatedTopicUrl;

    #[tracing::instrument(skip(document), fields(url = %url.to_string()))]
    fn from_scraped_page(url: &Self::Url, document: &Html) -> Result<Self> {
        let article = Self::extract_article(document).ok_or(BBCError::NoArticleFound {
            url: url.full_url(),
        })?;

        let title = Self::extract_title(&article).ok_or(BBCError::NoTitleFound {
            url: url.full_url(),
        })?;

        let content = Self::extract_content(&article).ok_or(BBCError::NoContentFound {
            url: url.full_url(),
        })?;

        let related_topics =
            Self::extract_related_topics(&article).ok_or(BBCError::NoRelatedTopicsFound {
                url: url.full_url(),
            })?;

        let timestamp = Self::extract_timestamp(&article);

        let page_links = Self::extract_related_links(&article);
        let page_links = Self::convert_to_page(page_links);

        Ok(BBCContent::new(
            title,
            content,
            url.clone(),
            related_topics,
            timestamp,
            page_links,
        ))
    }

    fn get_related_pages(&self) -> HashSet<Page<LinkTo, Self::Url>> {
        self.metadata.page_links.clone()
    }

    fn get_related_topics(&self) -> HashSet<RelatedPage<Self::RelatedUrl>> {
        self.metadata.related_topics.clone()
    }

    fn get_title(&self) -> String {
        self.title.clone()
    }

    fn get_url(&self) -> Self::Url {
        self.metadata.url.clone()
    }
}

impl BBCContent {
    fn new(
        title: String,
        content: Vec<String>,
        url: BBCNewsUrl,
        related_topics: HashSet<RelatedPage<BBCRelatedTopicUrl>>,
        timestamp: String,
        page_links: HashSet<Page<LinkTo, BBCNewsUrl>>,
    ) -> Self {
        BBCContent {
            title,
            content,
            metadata: Metadata {
                url,
                related_topics,
                timestamp,
                page_links,
            },
        }
    }

    fn extract_article(document: &Html) -> Option<ElementRef> {
        let article_selector = scraper::Selector::parse("article").unwrap();
        document.select(&article_selector).next()
    }

    fn extract_title(article: &ElementRef) -> Option<String> {
        let title_selector = scraper::Selector::parse("h1").unwrap();
        article
            .select(&title_selector)
            .next()
            .map(|title| title.text().collect::<String>())
    }
    fn extract_content(article: &ElementRef) -> Option<Vec<String>> {
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
    fn extract_related_topics(
        article: &ElementRef,
    ) -> Option<HashSet<RelatedPage<BBCRelatedTopicUrl>>> {
        let related_topics_selector =
            scraper::Selector::parse("div[data-component='topic-list']").unwrap();
        let related_topics = article.select(&related_topics_selector).next()?;

        let related_topics = Self::extract_related_links(&related_topics)
            .into_iter()
            .filter_map(|(url, title)| {
                let url = BBCRelatedTopicUrl::parse(url).ok()?;
                let title = title.clone();
                Some(RelatedPage::new(url, title))
            })
            .collect::<HashSet<RelatedPage<BBCRelatedTopicUrl>>>();

        Some(related_topics)
    }
    pub fn extract_related_links(article: &ElementRef) -> Vec<(String, String)> {
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
    pub fn convert_to_page(
        i: impl IntoIterator<Item = (String, String)>,
    ) -> HashSet<Page<LinkTo, BBCNewsUrl>> {
        i.into_iter()
            .filter_map(|(url, title)| {
                let url = BBCNewsUrl::try_from(url).ok()?;
                let title = title.clone();
                Some(Page::<LinkTo, BBCNewsUrl>::new(url, title))
            })
            .collect::<HashSet<Page<LinkTo, BBCNewsUrl>>>()
    }
    fn extract_timestamp(article: &ElementRef) -> String {
        let timestamp_selector = scraper::Selector::parse("time").unwrap();
        article
            .select(&timestamp_selector)
            .next()
            .and_then(|element| element.value().attr("datetime"))
            .unwrap_or_default()
            .to_string()
    }
}
