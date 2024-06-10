use crate::Result;

use super::{make_request, UrlTrait};

use scraper::Html;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::Debug;
use std::{rc::Rc, sync::Arc};

/// A trait for the state of a page.
pub trait PageState {
    /// Audit the page state. This is used for debugging. And simply to write the macro impl_page_state_and_as_ref!.
    fn audit(&self) -> String;
}
/// A struct representing a page that is yet to be scraped.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct ToScrape;
// A struct representing a link to another page. This is used to keep track of the links on a page. A LinkTo page is not scraped yet but can be.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct LinkTo {
    title: String,
}

/// A struct representing a page that has been scraped. The content field is the scraped content of the page.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Scraped<C: ScrapableContent> {
    content: C,
}

macro_rules! impl_page_state_and_as_ref {
    ($($state:ty),+) => {
        $(impl PageState for $state {
            fn audit(&self) -> String {
                format!("{:?}", self)
            }
        })+
    };
}

impl_page_state_and_as_ref!(ToScrape, LinkTo);

impl<C: ScrapableContent> PageState for Scraped<C> {
    fn audit(&self) -> String {
        format!("Scraped: {:?}", self)
    }
}

/// A struct representing a page. The state field is the state of the page. The url field is the URL of the page. The url field is an Arc because the URL is shared with the scraper.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Page<S: PageState, U: UrlTrait> {
    url: Arc<U>,
    state: S,
}

impl<S: PageState, U: UrlTrait> AsRef<U> for Page<S, U> {
    fn as_ref(&self) -> &U {
        &*self.url
    }
}

impl<S: PageState, U: UrlTrait> Page<S, U> {
    /// Transition to a new state.
    fn transition<N: PageState>(self, next: N) -> Page<N, U> {
        Page {
            url: self.url,
            state: next,
        }
    }
}

impl<U: UrlTrait> Page<ToScrape, U> {
    /// Create a new ToScrape page with a URL.
    pub fn new(url: U) -> Self {
        Page {
            url: Arc::new(url),
            state: ToScrape,
        }
    }
    /// Create a new ToScrape page with a URL.
    pub fn new_to_scrape(url: U) -> Self {
        Self::new(url)
    }
}

impl<U: UrlTrait> Page<LinkTo, U> {
    /// Create a new LinkTo page with a URL and a title.
    pub fn new(url: U, title: impl Into<String>) -> Self {
        Page {
            url: Arc::new(url),
            state: LinkTo {
                title: title.into(),
            },
        }
    }
    /// Create a new LinkTo page with a URL and a title.
    pub fn new_link_to(url: U, title: impl Into<String>) -> Self {
        Self::new(url, title)
    }
}

/// This is a trait that is used to represent a page state.
pub trait ScrapableContent: Debug {
    /// The type of the Url.
    type Url: UrlTrait;
    /// This is a helper method that takes a url and a document and returns a Result of the type.
    fn from_scraped_page(url: &Self::Url, document: &Html) -> Result<Self>
    where
        Self: Sized;
}

/// A trait for types that can be scraped. This means they can be converted into a Scraped type where the content is the scraped content.
pub trait Scrapable {}

impl Scrapable for ToScrape {}
impl Scrapable for LinkTo {}

impl<U: UrlTrait, S: PageState + Scrapable> Page<S, U> {
    /// Scrape the page. This will make a request to the page and scrape the content. The content is then converted into a Scraped type.
    pub async fn scrape<C: ScrapableContent<Url = U>>(self) -> Result<Page<Scraped<C>, U>>
    where
        C: ScrapableContent<Url = U>,
    {
        let url = self.url.as_ref();
        let html = make_request(url).await?;
        let page = C::from_scraped_page(&url, &html)?;

        Ok(self.transition(Scraped { content: page }))
    }
}
