use crate::common::{MockDB, Page, PageHandler, Scrapable, ScrapableContent, UrlTrait, DB};

use std::cmp::Eq;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Scraper<U, C>
where
    U: UrlTrait + Eq,
    C: ScrapableContent<Url = U>,
{
    page_handler: PageHandler<U>,
    db_service: DB<C>,
}

impl<U, C> Scraper<U, C>
where
    U: UrlTrait,
    C: ScrapableContent<Url = U>,
{
    pub fn new(db_service: DB<C>) -> Self {
        Self {
            page_handler: PageHandler::<U>::new(),
            db_service,
        }
    }

    pub async fn add_page(&mut self, page: Box<Page<dyn Scrapable, U>>) {
        self.page_handler.add_page(page).await;
    }

    pub async fn add_pages<I>(&mut self, pages: I)
    where
        I: IntoIterator<Item = Box<Page<dyn Scrapable, U>>> + Send,
    {
        self.page_handler.add_pages(pages).await;
    }

    pub async fn scrape_pages_recursive(&mut self, max_depth: u32) {
        self.page_handler
            .scrape_pages_recursive::<C>(self.db_service.clone(), max_depth)
            .await;
    }

    pub fn get_db(&self) -> DB<C> {
        Arc::clone(&self.db_service)
    }
}

impl<U, C> Default for Scraper<U, C>
where
    U: UrlTrait,
    C: ScrapableContent<Url = U> + 'static,
{
    fn default() -> Self {
        let db_service = Arc::new(Mutex::new(MockDB::<C>::default()));

        Self::new(db_service)
    }
}
