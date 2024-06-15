use crate::Result;

use super::{LinkTo, Page, UrlTrait};

use scraper::Html;
use std::collections::HashSet;
use std::fmt::Debug;

/// This is a trait that is used to represent a page state.
pub trait ScrapableContent: Debug + Eq + Send {
    /// The type of the Url.
    type Url: UrlTrait;
    /// This is a helper method that takes a url and a document and returns a Result of the type.
    fn from_scraped_page(url: &Self::Url, document: &Html) -> Result<Self>
    where
        Self: Sized;

    fn get_related_pages(&self) -> HashSet<Page<LinkTo, Self::Url>>;
}
