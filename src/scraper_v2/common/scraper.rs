use crate::Result;

use crate::common::{
    DatabaseService, Page, PageHandler, Scrapable, ScrapableContent, UrlTrait, DB,
};

use async_trait::async_trait;
use std::sync::Arc;
use tracing::instrument;

#[async_trait]
pub trait SiteSpecificScraper {
    type Url: UrlTrait;

    async fn scrape_initial_urls(&self) -> Result<Vec<Box<Page<dyn Scrapable, Self::Url>>>>;
}

#[derive(Debug, Clone)]
pub struct Scraper<S: SiteSpecificScraper> {
    site_scraper: S,
    page_handler: PageHandler<S::Url>,
}

impl<S: SiteSpecificScraper> Scraper<S> {
    pub fn new(site_scraper: S) -> Self {
        Self {
            site_scraper,
            page_handler: PageHandler::<S::Url>::new(),
        }
    }

    pub async fn add_page(&mut self, page: Box<Page<dyn Scrapable, S::Url>>) {
        self.page_handler.add_page(page).await;
    }

    pub async fn add_pages<I>(&mut self, pages: I)
    where
        I: IntoIterator<Item = Box<Page<dyn Scrapable, S::Url>>> + Send,
    {
        self.page_handler.add_pages(pages).await;
    }

    pub async fn scrape_pages_recursive<
        C: ScrapableContent<Url = S::Url> + 'static,
        D: DatabaseService,
    >(
        &mut self,
        db_service: &DB<D>,
        max_depth: u32,
    ) {
        self.page_handler
            .scrape_pages_recursive::<C, D>(Arc::clone(db_service), max_depth)
            .await;
    }

    #[instrument(skip_all, name = "Initialize Scraper", level = "info")]
    pub async fn initialize(&mut self) -> Result<()> {
        let initial_urls = self.site_scraper.scrape_initial_urls().await?;
        self.page_handler.add_pages(initial_urls).await;
        Ok(())
    }
}
