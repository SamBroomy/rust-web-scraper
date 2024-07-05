use crate::Result;

use crate::common::{make_request, DatabaseService, ScrapableContent, UrlTrait, DB};

use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::{error, field, info, instrument, warn, Span};

/// A trait for the state of a page.
pub trait PageState: Debug + Send + Sync {
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
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
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
        &self.url
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
    #[instrument(
        skip_all,
        name = "Scraped Page",
        level = "info",
        fields(linked_article)
    )]
    pub async fn scrape<C>(self) -> Result<Page<WasScraped<C>, U>>
    where
        C: ScrapableContent<Url = U>,
    {
        let title = self.state.get_title();
        tracing::Span::current().record(
            "linked_article",
            title.as_ref().unwrap_or(&"[Scraping from URL]".to_string()),
        );
        let url = self.url.as_ref();
        let html = make_request(url).await?;
        let page = C::from_scraped_page(url, &html)?;

        Ok(self.transition(WasScraped {
            content: page,
            link_title: title,
        }))
    }
}
impl<U: UrlTrait, S: Scrapable + ?Sized> Page<S, U> {
    /// Scrape the page. This will make a request to the page and scrape the content. The content is then converted into a Scraped type.
    /// Would prefer if this was consuming self but it's not possible because of the transition method.
    #[instrument(
        skip_all,
        name = "Scraped Page (Box)",
        level = "info",
        fields(linked_article)
    )]
    pub async fn scrape_in_place<C>(self: Box<Self>) -> Result<Page<WasScraped<C>, U>>
    where
        C: ScrapableContent<Url = U>,
    {
        let title = self.state.get_title();
        tracing::Span::current().record(
            "linked_article",
            title.as_ref().unwrap_or(&"[Scraping from URL]".to_string()),
        );
        let url = self.url.as_ref();
        let html = make_request(url).await?;
        let page = C::from_scraped_page(url, &html)?;
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

    pub fn get_content(&self) -> &C {
        &self.state.content
    }

    pub fn get_title(&self) -> Option<String> {
        self.state.link_title.clone()
    }
}

pub type ScrapablePagesQueue<U> = Arc<Mutex<VecDeque<Box<Page<dyn Scrapable, U>>>>>;

#[derive(Debug, Clone, Default)]
pub struct PageHandler<U: UrlTrait> {
    visited: Arc<Mutex<HashSet<Arc<U>>>>,
    pages_queue: ScrapablePagesQueue<U>,
}

impl<U: UrlTrait> PageHandler<U>
where
    U: UrlTrait,
{
    pub fn new() -> Self {
        Self {
            visited: Arc::new(Mutex::new(HashSet::new())),
            pages_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    #[instrument(skip_all, name = "Add Page", level = "info", fields(page = %page.get_url_arc().to_string()))]
    pub async fn add_page(&mut self, page: Box<Page<dyn Scrapable, U>>) {
        let mut pages_queue = self.pages_queue.lock().await;
        pages_queue.push_back(page);
    }

    #[instrument(skip_all, name = "Add Pages", level = "info", fields(pages))]
    pub async fn add_pages<I>(&mut self, pages: I)
    where
        I: IntoIterator<Item = Box<Page<dyn Scrapable, U>>> + Send,
    {
        let mut pages_list = self.pages_queue.lock().await;
        let pages_len = pages_list.len();

        pages_list.extend(pages);

        tracing::Span::current().record("pages", pages_list.len() - pages_len);
    }

    #[instrument(skip(self, db), name = "Scrape Pages Recursive", level = "warn")]
    pub async fn scrape_pages_recursive<
        C: ScrapableContent<Url = U> + 'static,
        D: DatabaseService,
    >(
        &mut self,
        db: DB<D>,
        mut max_depth: u32,
    ) {
        assert!(max_depth > 0, "Max depth must be greater than 0!");
        if max_depth > 10 {
            warn!("Max depth is too high, setting to 10!");
            max_depth = 10;
        }
        let max_depth = max_depth;

        warn!(
            "Scraping pages recursively with max depth: {:#?}",
            max_depth
        );

        let parent_span = Span::current();

        self.get_pages_recursive_internal::<C, D>(db, max_depth, 0, parent_span)
            .await;
    }

    /// Drain all pages from the queue.
    //? Trying to keep the scope of the lock as small as possible.
    async fn drain_pages(&mut self) -> Vec<Box<Page<dyn Scrapable, U>>> {
        let mut pages_queue = self.pages_queue.lock().await;
        pages_queue.drain(..).collect()
    }

    fn get_unique_pages_to_scrape(
        &self,
        pages_to_scrape: Vec<Box<Page<dyn Scrapable, U>>>,
    ) -> Vec<Box<Page<dyn Scrapable, U>>> {
        let mut seen = HashSet::new();

        pages_to_scrape
            .into_iter()
            .filter(|page| seen.insert(page.get_url_arc()))
            .collect::<Vec<Box<Page<dyn Scrapable, U>>>>()
    }

    #[instrument(
        skip(self),
        name = "Extract Unique Pages",
        level = "info",
        fields(pages_to_scrape, unique_pages, duplicates_dropped, not_visited)
    )]
    async fn extract_unique_pages_to_scrape(&mut self) -> Vec<Box<Page<dyn Scrapable, U>>> {
        let pages_to_scrape = self.drain_pages().await;
        let len_pages_to_scrape = pages_to_scrape.len();
        Span::current().record("pages_to_scrape", len_pages_to_scrape);

        let mut unique_pages_to_scrape = self.get_unique_pages_to_scrape(pages_to_scrape);
        let len_unique_pages_to_scrape = unique_pages_to_scrape.len();
        Span::current().record("unique_pages", len_unique_pages_to_scrape);

        Span::current().record(
            "duplicates_dropped",
            len_pages_to_scrape - len_unique_pages_to_scrape,
        );

        self.remove_visited_pages(&mut unique_pages_to_scrape).await;

        Span::current().record(
            "not_visited",
            len_unique_pages_to_scrape - unique_pages_to_scrape.len(),
        );

        unique_pages_to_scrape
    }

    /// Remove pages that have already been visited from the list of pages to scrape.
    ///? Again, trying to keep the scope of the lock as small as possible.
    async fn remove_visited_pages(&self, pages_to_scrape: &mut Vec<Box<Page<dyn Scrapable, U>>>) {
        let visited = self.visited.lock().await;
        pages_to_scrape.retain(|page| !visited.contains(&page.get_url_arc()));
    }

    async fn get_pages_recursive_internal<
        C: ScrapableContent<Url = U> + 'static,
        D: DatabaseService,
    >(
        &mut self,
        db: DB<D>,
        max_depth: u32,
        current_depth: u32,
        parent_span: Span,
    ) {
        let start_time = Instant::now();
        let span = tracing::warn_span!(
            parent: &parent_span,
            "Recursive Call",
            depth = current_depth,
            pages_to_scrape = field::Empty,
            unique_pages = field::Empty,
        );

        let _enter = span.enter();

        if current_depth > max_depth {
            warn!("Max depth reached!");
            return;
        }
        warn!(
            " ------------------------------------- ITERATION {} -------------------------------------",
            current_depth);

        let unique_pages_to_scrape = self.extract_unique_pages_to_scrape().await;

        stream::iter(unique_pages_to_scrape.into_iter())
            .for_each_concurrent(None, |scrapable_page| {
                //? Each of the tasks need to have access to scraped_pages, but cant directly pass scraped_pages to them because it would mean multiple owners. What we are doing here is creating a new reference (Arc) to the data (.clone()). This new arc can then be moved into the concurrent task, giving it access to the shared data.
                let visited_mutex = Arc::clone(&self.visited);
                let pages_mutex = Arc::clone(&self.pages_queue);

                let db = Arc::clone(&db);

                //? here the async means creating an async block of code that can be awaited.
                //? The move means the closure takes ownership of the values it uses inside the closure (url, scraped_pages).
                async move {
                    //? Lock the visited_urls, check if the url has been visited, if it has return.
                    {
                        let visited_urls = visited_mutex.lock().await;
                        if visited_urls.contains(&scrapable_page.get_url_arc()) {
                            return;
                        }
                    }

                    if let Ok(page) = scrapable_page.scrape_in_place::<C>().await {
                        let linked_pages = page
                            .get_all_page_links()
                            .into_iter()
                            .map(|page| Box::new(page) as Box<Page<dyn Scrapable, U>>)
                            .collect::<Vec<Box<Page<dyn Scrapable, U>>>>();

                        //? Lock and modify pages_to_scrape, then immediately drop the lock
                        {
                            let mut locked_pages_to_scrape = pages_mutex.lock().await;
                            locked_pages_to_scrape.extend(linked_pages);
                        } //? locked_pages_to_scrape is dropped here, releasing the lock
                          //? Didn't need to do the same thing here as the guard is dropped at the end of the block
                        {
                            let mut visited_urls = visited_mutex.lock().await;
                            visited_urls.insert(page.get_url_arc());
                        }

                        if let Err(e) = db.lock().await.save_content(page.get_content()).await {
                            error!("Error saving content: {:#?}", e);
                        }
                    }
                }
            })
            .await;

        info!("Pages visited: {:#?}", self.visited.lock().await.len());

        drop(_enter);

        Box::pin(self.get_pages_recursive_internal::<C, D>(
            db,
            max_depth,
            current_depth + 1,
            parent_span.clone(),
        ))
        .await;

        // Re-enter the span to log total time
        let _reenter = span.enter();
        let total_elapsed = start_time.elapsed();
        warn!(
            total_time_spent_ms = total_elapsed.as_millis(),
            "Total time spent including child recursions at depth {:#}", current_depth
        );
    }
}
