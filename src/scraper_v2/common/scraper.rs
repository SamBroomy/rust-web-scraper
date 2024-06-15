use crate::common::{Page, PageState, Scrapable, ScrapableContent, UrlTrait, WasScraped};
use crate::Result;

use async_trait::async_trait;
use futures::stream::futures_unordered::IntoIter;
use futures::stream::{self, StreamExt};
use std::cmp::Eq;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::Scraped;

// TODO: This should be in model? Implement DatabaseService for your specific database
#[async_trait]
pub trait DatabaseService<C> {
    async fn save_content(&self, content: &C) -> Result<()>;
    // Additional methods for link management or querying could be added here
}

// struct MyDatabaseService; // Implement DatabaseService for your specific database

// impl<C> DatabaseService<C> for MyDatabaseService {
//     async fn save_content(&self, content: &C) -> Result<()> {
//         // Save content to database
//         Ok(())
//     }
// }

#[async_trait]
pub trait PageScraper<U: UrlTrait> {
    // fn new(db_service: Arc<dyn DatabaseService<C>>) -> Self
    // where
    //     Self: Sized;

    async fn add_page(&mut self, page: Box<Page<dyn Scrapable, U>>);
    async fn add_pages<I: IntoIterator<Item = Box<Page<dyn Scrapable, U>>> + Send>(
        &mut self,
        page: I,
    );
    async fn scrape_pages_recursive<C: ScrapableContent<Url = U>>(&mut self, mut max_depth: u32);
}

#[derive(Debug)]
pub struct PageHandler<U: UrlTrait> {
    //scraper: Box<dyn Scraper<U, C>>,
    visited: Arc<Mutex<HashSet<Arc<U>>>>,
    pages_queue: Arc<Mutex<VecDeque<Box<Page<dyn Scrapable, U>>>>>,
}

#[async_trait]
impl<U> PageScraper<U> for PageHandler<U>
where
    U: UrlTrait + std::marker::Sync + std::marker::Send,
{
    async fn add_page(&mut self, page: Box<Page<dyn Scrapable, U>>) {
        let mut pages_queue = self.pages_queue.lock().await;
        pages_queue.push_back(page);
    }
    async fn add_pages<I>(&mut self, pages: I)
    where
        I: IntoIterator<Item = Box<Page<dyn Scrapable, U>>> + Send,
    {
        let mut pages_list = self.pages_queue.lock().await;
        pages_list.extend(pages);
    }

    async fn scrape_pages_recursive<C: ScrapableContent<Url = U>>(&mut self, mut max_depth: u32) {
        if max_depth > 10 {
            println!("Max depth is too high, setting to 10!");
            max_depth = 10;
        }
        let max_depth = max_depth;
        self.get_pages_recursive_internal::<C>(max_depth, 0).await;
    }
}

impl<U: UrlTrait> PageHandler<U>
where
    U: UrlTrait + Eq,
    //S: PageState + Scrapable + Eq,
    //C: ScrapableContent<Url = U>,
{
    pub fn new(/*scraper: Box<dyn Scraper<U, C>>*/) -> Self {
        Self {
            //scraper,
            visited: Arc::new(Mutex::new(HashSet::new())),
            pages_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
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

        let unique_pages_to_scrape = pages_to_scrape
            .into_iter()
            .filter(|page| seen.insert(page.get_url_arc()))
            .collect::<Vec<Box<Page<dyn Scrapable, U>>>>();
        unique_pages_to_scrape
    }

    /// Remove pages that have already been visited from the list of pages to scrape.
    ///? Again, trying to keep the scope of the lock as small as possible.
    async fn remove_visited_pages(&self, pages_to_scrape: &mut Vec<Box<Page<dyn Scrapable, U>>>) {
        let visited = self.visited.lock().await;
        pages_to_scrape.retain(|page| !visited.contains(&page.get_url_arc()));
    }

    async fn get_pages_recursive_internal<C: ScrapableContent<Url = U>>(
        &mut self,
        max_depth: u32,
        current_depth: u32,
    ) {
        if current_depth > max_depth {
            return;
        }

        println!(
        " ------------------------------------- ITERATION {} -------------------------------------",
        current_depth);

        // if self.pages.is_empty() {
        //     println!("No more pages to visit!");
        //     return Ok(());
        // } else {
        //     println!(
        //         "Number of pages to visit this iteration: {:?}",
        //         self.pages.len()
        //     );
        // }

        let mut pages_to_scrape = self.drain_pages().await;

        println!("Pages to scrape: {:#?}", pages_to_scrape);

        let mut unique_pages_to_scrape = self.get_unique_pages_to_scrape(pages_to_scrape);

        println!("Unique Pages: {:#?}", unique_pages_to_scrape);

        self.remove_visited_pages(&mut unique_pages_to_scrape).await;

        println!(
            "Pages to scrape after removing visited pages: {:#?}",
            unique_pages_to_scrape
        );

        stream::iter(unique_pages_to_scrape.into_iter())
            .for_each_concurrent(None, |scrapable_page| {
                //? Each of the tasks need to have access to scraped_pages, but cant directly pass scraped_pages to them because it would mean multiple owners. What we are doing here is creating a new reference (Arc) to the data (.clone()). This new arc can then be moved into the concurrent task, giving it access to the shared data.
                let visited_mutex = Arc::clone(&self.visited);
                let pages_mutex = Arc::clone(&self.pages_queue);

                //? here the async means creating an async block of code that can be awaited.
                //? The move means the closure takes ownership of the values it uses inside the closure (url, scraped_pages).
                async move {
                    {
                        let mut visited_urls = visited_mutex.lock().await;
                        if visited_urls.contains(&scrapable_page.get_url_arc()) {
                            return;
                        }
                    }

                    if let Some(page) = scrapable_page.scrape_in_place::<C>().await.ok() {
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
                        let mut visited_urls = visited_mutex.lock().await;
                        visited_urls.insert(page.get_url_arc());
                        // TODO: Insert data to db here?
                    }
                }
            })
            .await;

        println!("Pages visited: {:#?}", self.visited.lock().await.len());

        Box::pin(self.get_pages_recursive_internal::<C>(max_depth, current_depth + 1)).await;
    }
}
