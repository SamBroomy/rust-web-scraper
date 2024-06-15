use crate::Result;

use super::{make_request, UrlTrait};

use scraper::Html;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::{rc::Rc, sync::Arc};

/// A trait for the state of a page.
pub trait PageState: Debug + Send {
    // Eq and Hash are required for HashSet
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
pub struct WasScraped<C: ScrapableContent> {
    content: C,
    link_title: Option<String>,
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

impl<C: ScrapableContent> PageState for WasScraped<C> {
    fn audit(&self) -> String {
        format!("Scraped: {:?}", self)
    }
}

/// A struct representing a page. The state field is the state of the page. The url field is the URL of the page. The url field is an Arc because the URL is shared with the scraper.
///
/// ## Self Note - ?Sized
///
/// ?Sized is used to relax the Sized trait bound. By default, all generic parameters have the Sized trait bound, which means they must have a compile-time known size. The ?Sized bound allows for types that do not have a known size at compile time, such as slices and trait objects.
///
/// The reason we need to use ?Sized is because in the Scraper we want a list of Scrapable pages (ToScrape and LinkTo). We can't have a list of Scrapable because Scrapable is a trait and doesn't have a known size at compile time. So we need to use ?Sized to relax the Sized trait bound.
///
/// The following code would now not work:
/// ```rust
/// let page: Page<dyn PageState, Url> = Page {
///     url: Arc::new(Url::parse("https://example.com").unwrap()),
///     state: ToScrape,
/// };
/// ```
/// This is because dyn PageState is a trait object and does not have a known size at compile time. The Page struct requires its state field to be Sized, so this code will not compile.
///
/// However, you can still create a Page instance if S is Sized. For example:
/// ```rust
///
/// let page: Page<ToScrape, Url> = Page {
///     url: Arc::new(Url::parse("https://example.com").unwrap()),
///     state: ToScrape,
/// };
/// ```
///
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Page<S: PageState + ?Sized, U: UrlTrait> {
    url: Arc<U>,
    state: S,
}

impl<S: PageState, U: UrlTrait> Page<S, U> {
    /// Transition to a new state.
    fn transition<N: PageState>(self, next: N) -> Page<N, U> {
        Page {
            url: Arc::clone(&self.url),
            state: next,
        }
    }
}
impl<S: PageState + ?Sized, U: UrlTrait> Page<S, U> {
    pub fn get_url_arc(&self) -> Arc<U> {
        Arc::clone(&self.url)
    }
    /// Transition to a new state while keeping the page in place or in a box.
    fn transition_in_place<N: PageState>(self: Box<Self>, next: N) -> Box<Page<N, U>> {
        Box::new(Page {
            url: Arc::clone(&self.url),
            state: next,
        })
    }
}

/// Hash implementation for Page. It hashes the URL of the page. The hash of the UrlTrait will be hashed on the url string.
impl<S: PageState, U: UrlTrait> Hash for Page<S, U> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.url.hash(state);
    }
}

impl<S: PageState, U: UrlTrait> AsRef<U> for Page<S, U> {
    /// Get a reference to the URL of the page.
    fn as_ref(&self) -> &U {
        &*self.url
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

/// A trait for types that can be scraped. This means they can be converted into a Scraped type where the content is the scraped content.
pub trait Scrapable: PageState {
    fn get_title(&self) -> Option<String> {
        None
    }
}

impl Scrapable for ToScrape {}
impl Scrapable for LinkTo {
    fn get_title(&self) -> Option<String> {
        Some(self.title.clone())
    }
}

impl<U: UrlTrait, S: Scrapable> Page<S, U> {
    /// Scrape the page. This will make a request to the page and scrape the content. The content is then converted into a Scraped type.
    pub async fn scrape<C: ScrapableContent<Url = U>>(self) -> Result<Page<WasScraped<C>, U>>
    where
        C: ScrapableContent<Url = U>,
    {
        let title = self.state.get_title();
        let url = self.url.as_ref();
        let html = make_request(url).await?;
        let page = C::from_scraped_page(&url, &html)?;

        Ok(self.transition(WasScraped {
            content: page,
            link_title: title,
        }))
    }
}

impl<U: UrlTrait, S: Scrapable + ?Sized> Page<S, U> {
    /// Scrape the page. This will make a request to the page and scrape the content. The content is then converted into a Scraped type.
    /// Would prefer if this was consuming self but it's not possible because of the transition method.
    pub async fn scrape_in_place<C: ScrapableContent<Url = U>>(
        self: Box<Self>,
    ) -> Result<Page<WasScraped<C>, U>>
    where
        C: ScrapableContent<Url = U>,
    {
        let title = self.state.get_title();
        let url = self.url.as_ref();
        let html = make_request(url).await?;
        let page = C::from_scraped_page(&url, &html)?;
        // Because we are going from a unsized type to a sized type, we can take the data out of the box and put it back on the stack.
        Ok(*self.transition_in_place(WasScraped {
            content: page,
            link_title: title,
        }))
    }
}

pub trait Scraped: PageState {}
impl<C: ScrapableContent> Scraped for WasScraped<C> {}

impl<U, C> Page<WasScraped<C>, U>
where
    U: UrlTrait,
    C: ScrapableContent<Url = U>,
{
    pub fn get_all_page_links(&self) -> HashSet<Page<LinkTo, U>> {
        self.state.content.get_related_pages()
    }
}
