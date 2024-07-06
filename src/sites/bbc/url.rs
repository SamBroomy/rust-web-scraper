use crate::common::UrlTrait;
use crate::{Error, Result};

use super::error::BBCError;

use lazy_regex::regex_is_match;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use tracing::{instrument, Span};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct BBCNewsUrl(String);

impl AsRef<String> for BBCNewsUrl {
    fn as_ref(&self) -> &String {
        &self.0
    }
}
impl TryFrom<String> for BBCNewsUrl {
    type Error = Error;
    fn try_from(url: String) -> Result<Self> {
        match Self::parse_url(&url) {
            Ok(url) => Ok(BBCNewsUrl(url)),
            Err(e) => Err(e),
        }
    }
}

impl From<BBCNewsUrl> for String {
    fn from(url: BBCNewsUrl) -> String {
        url.0
    }
}

impl Hash for BBCNewsUrl {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl UrlTrait for BBCNewsUrl {
    fn base_url() -> &'static str {
        "https://www.bbc.co.uk"
    }
    fn to_string(&self) -> String {
        self.0.clone()
    }

    fn initialise() -> Self {
        Self("/news".to_string())
    }

    #[instrument(
        skip_all,
        name = "Parse BBC URL",
        level = "trace",
        err(level = "trace"),
        fields(stripped_url)
    )]
    fn parse_url(url: &str) -> Result<String> {
        let stripped_url = url.trim().strip_prefix(Self::base_url()).unwrap_or(url);
        let stripped_url = stripped_url.split('#').next().unwrap_or(stripped_url);
        let stripped_url = stripped_url.split('?').next().unwrap_or(stripped_url);

        Span::current().record("stripped_url", &stripped_url);

        if !stripped_url.starts_with("/news") || regex_is_match!(r"/(Special:|File:)", stripped_url)
        {
            let reason = {
                if !stripped_url.starts_with("/news") {
                    "Does not start with /news"
                } else {
                    "Contains Special: or File:"
                }
                .to_string()
            };

            return Err(BBCError::InvalidUrl {
                url: url.to_string(),
                reason,
            }
            .into());
        };

        let segments: Vec<&str> = stripped_url.split('/').collect();
        let ends_with_number = segments.last().map_or(false, |last_segment| {
            last_segment.split('-').last().map_or(false, |last_word| {
                last_word.len() == 8 && last_word.parse::<u32>().is_ok()
            })
        });
        let has_article_or_number = segments
            .get(2)
            .map_or(false, |&segment| segment == "articles" || ends_with_number);
        let contains_live_segment = segments.iter().any(|&segment| segment == "live");

        if !has_article_or_number || contains_live_segment {
            let reason = if !has_article_or_number {
                "Does not contain article or number"
            } else {
                "Contains live segment"
            }
            .to_string();
            Err(BBCError::InvalidUrl {
                url: url.to_string(),
                reason,
            }
            .into())
        } else {
            Ok(stripped_url.to_string())
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct BBCRelatedTopicUrl(String);

impl AsRef<String> for BBCRelatedTopicUrl {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl TryFrom<String> for BBCRelatedTopicUrl {
    type Error = Error;
    fn try_from(url: String) -> Result<Self> {
        match Self::parse_url(&url) {
            Ok(url) => Ok(BBCRelatedTopicUrl(url)),
            Err(e) => Err(e),
        }
    }
}

impl From<BBCRelatedTopicUrl> for String {
    fn from(url: BBCRelatedTopicUrl) -> String {
        url.0
    }
}

impl Hash for BBCRelatedTopicUrl {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl UrlTrait for BBCRelatedTopicUrl {
    fn base_url() -> &'static str {
        "https://www.bbc.co.uk"
    }
    fn to_string(&self) -> String {
        self.0.clone()
    }

    fn initialise() -> Self {
        Self("/news".to_string())
    }

    #[instrument(
        skip_all,
        name = "Parse BBC URL",
        level = "trace",
        err(level = "trace"),
        fields(stripped_url)
    )]
    fn parse_url(url: &str) -> Result<String> {
        let stripped_url = url.trim().strip_prefix(Self::base_url()).unwrap_or(url);
        let stripped_url = stripped_url.split('#').next().unwrap_or(stripped_url);
        let stripped_url = stripped_url.split('?').next().unwrap_or(stripped_url);

        Span::current().record("stripped_url", &stripped_url);

        if !stripped_url.starts_with("/news") || regex_is_match!(r"/(Special:|File:)", stripped_url)
        {
            let reason = {
                if !stripped_url.starts_with("/news") {
                    "Does not start with /news"
                } else {
                    "Contains Special: or File:"
                }
                .to_string()
            };

            return Err(BBCError::InvalidUrl {
                url: url.to_string(),
                reason,
            }
            .into());
        };

        let segments: Vec<&str> = stripped_url.split('/').collect();
        let ends_with_number = segments.last().map_or(false, |last_segment| {
            last_segment.split('-').last().map_or(false, |last_word| {
                last_word.len() == 8 && last_word.parse::<u32>().is_ok()
            })
        });
        let has_article_or_number = segments
            .get(2)
            .map_or(false, |&segment| segment == "topics" || ends_with_number);
        let contains_live_segment = segments.iter().any(|&segment| segment == "live");

        if !has_article_or_number || contains_live_segment {
            let reason = if !has_article_or_number {
                "Does not contain topics or number"
            } else {
                "Contains live segment"
            }
            .to_string();
            Err(BBCError::InvalidUrl {
                url: url.to_string(),
                reason,
            }
            .into())
        } else {
            Ok(stripped_url.to_string())
        }
    }
}
