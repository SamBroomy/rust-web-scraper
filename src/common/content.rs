use crate::Result;

use crate::common::{LinkTo, Page, RelatedPage, UrlTrait};

use scraper::Html;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::Debug;

/// This is a trait that is used to represent a page state.
pub trait ScrapableContent:
    Debug + Send + Sync + Clone + Serialize + for<'de> Deserialize<'de>
{
    /// The type of the Url.
    type Url: UrlTrait;
    type RelatedUrl: UrlTrait + DeserializeOwned;

    /// This is a helper method that takes a url and a document and returns a Result of the type.
    fn from_scraped_page(url: &Self::Url, document: &Html) -> Result<Self>
    where
        Self: Sized;

    fn get_related_pages(&self) -> HashSet<Page<LinkTo, Self::Url>>;
    fn get_related_topics(&self) -> HashSet<RelatedPage<Self::RelatedUrl>>;

    fn get_title(&self) -> String;
    fn get_url(&self) -> Self::Url;
}
