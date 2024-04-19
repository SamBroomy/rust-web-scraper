use lazy_regex::regex_is_match;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
pub trait UrlTrait: Hash {
    fn new(url: impl Into<String>) -> Option<Self>
    where
        Self: Sized;
    fn base_url() -> &'static str;
    fn specific_path(&self) -> String;
    fn full_url(&self) -> String {
        format!("{}{}", Self::base_url(), self.specific_path())
    }
    fn parse_url(url: String) -> Option<String>;
    fn from_list(urls: Vec<String>) -> Vec<Self>
    where
        Self: Sized,
    {
        urls.into_iter().filter_map(Self::new).collect()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BBCUrl(pub String);

impl UrlTrait for BBCUrl {
    fn new(url: impl Into<String>) -> Option<Self> {
        Self::parse_url(url.into()).map(Self)
    }

    fn base_url() -> &'static str {
        "https://www.bbc.co.uk"
    }

    fn specific_path(&self) -> String {
        self.0.clone()
    }

    fn parse_url(url: String) -> Option<String> {
        let stripped_url = url.strip_prefix(Self::base_url()).unwrap_or(&url);
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
            Some(stripped_url.to_string())
        } else {
            None
        }
    }
}
impl Hash for BBCUrl {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WikipediaUrl(String);

impl UrlTrait for WikipediaUrl {
    fn new(url: impl Into<String>) -> Option<Self> {
        let url = url.into();
        Self::parse_url(url.into()).map(Self)
    }

    fn base_url() -> &'static str {
        "https://en.wikipedia.org"
    }

    fn specific_path(&self) -> String {
        self.0.clone()
    }

    fn parse_url(url: String) -> Option<String> {
        let stripped_url = url.strip_prefix(Self::base_url()).unwrap_or(&url);

        if !stripped_url.starts_with("/wiki") || regex_is_match!(r"/(Special:|File:)", stripped_url)
        {
            return None;
        }

        Some(stripped_url.to_string())
    }
}
impl Hash for WikipediaUrl {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Hash)]
pub enum Url {
    BBC(BBCUrl),
    Wikipedia(WikipediaUrl),
}
impl Url {
    pub fn url(&self) -> String {
        match self {
            Url::BBC(url) => url.full_url(),
            Url::Wikipedia(url) => url.full_url(),
        }
    }
}

impl From<BBCUrl> for Url {
    fn from(url: BBCUrl) -> Self {
        Url::BBC(url)
    }
}
impl From<WikipediaUrl> for Url {
    fn from(url: WikipediaUrl) -> Self {
        Url::Wikipedia(url)
    }
}
