use crate::common::UrlTrait;
use crate::{Error, Result};

use super::error::WikipediaError;
use super::WikipediaContent;

use lazy_regex::regex_is_match;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct WikipediaUrl(String);

impl AsRef<String> for WikipediaUrl {
    fn as_ref(&self) -> &String {
        &self.0
    }
}
impl TryFrom<String> for WikipediaUrl {
    type Error = Error;
    fn try_from(url: String) -> Result<Self> {
        match Self::parse_url(&url) {
            Ok(url) => Ok(WikipediaUrl(url)),

            Err(e) => Err(e),
        }
    }
}

impl From<WikipediaUrl> for String {
    fn from(url: WikipediaUrl) -> String {
        url.0
    }
}

impl Hash for WikipediaUrl {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl UrlTrait for WikipediaUrl {
    type ContentType = WikipediaContent;

    fn base_url() -> &'static str {
        "https://en.wikipedia.org"
    }
    fn to_string(&self) -> String {
        self.0.clone()
    }
    fn parse_url(url: &str) -> Result<String> {
        let stripped_url = url.strip_prefix(Self::base_url()).unwrap_or(url);

        if !stripped_url.starts_with("/wiki") || regex_is_match!(r"/(Special:|File:)", stripped_url)
        {
            let reason = {
                if !stripped_url.starts_with("/wiki") {
                    "Does not start with /wiki"
                } else {
                    "Contains Special: or File:"
                }
            }
            .to_string();

            return Err(WikipediaError::InvalidUrl {
                url: url.to_string(),
                reason,
            }
            .into());
        }

        Ok(stripped_url.to_string())
    }
}
