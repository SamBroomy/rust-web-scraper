use core::hash;
use std::collections::{HashSet, VecDeque};
use std::mem;
use std::rc::Rc;
use std::sync::Arc;

use my_crate::get_db::get_db;
use my_crate::scraper_v2::common::{
    make_request, LinkTo, Page, PageState, Scrapable, Scraped, ToScrape, UrlTrait, WasScraped,
};
use my_crate::scraper_v2::sites::bbc::{BBCContent, BBCUrl};
use my_crate::scraper_v2::Result;

use futures::stream::futures_unordered::IntoIter;
use futures::stream::{self, StreamExt};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tokio::sync::Mutex;
//use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    //let db = get_db("scraping").await?;

    let url = BBCUrl::parse("https://www.bbc.co.uk/news/articles/ceddenl8xz4o")?;
    println!("{:#?}", url);
    let page1 = Page::new_to_scrape(url);
    println!("{:#?}", page1);
    let page1 = page1.scrape::<BBCContent>().await?;
    println!("{:#?}", page1);

    // let url = BBCUrl::new("https://www.bbc.co.uk/news/articles/c8009e2z4xlo")?;
    // println!("{:#?}", url);
    // let page2 = Page::new_to_scrape(url).scrape::<BBCContent>().await?;
    // println!("{:#?}", page2);

    pub struct PageHandler<U: UrlTrait /* , C: ScrapableContent<Url = U>*/> {
        //scraper: Box<dyn Scraper<U, C>>,
        visited: HashSet<Rc<U>>,
        pages: VecDeque<Box<Page<dyn Scrapable, U>>>,
        // In Rust, when you use a trait as a type, you need to use it behind a pointer, like Box<dyn Trait>, &Trait, or Rc<dyn Trait>. This is because traits can be implemented by many different types with different sizes and Rust needs to know the size of a type at compile time.
    }

    impl<U /*, C*/> PageHandler<U /*, C*/>
    where
        U: UrlTrait + Eq,
        //C: ScrapableContent<Url = U>,
    {
        pub fn new(/*scraper: Box<dyn Scraper<U, C>>*/) -> Self {
            Self {
                //scraper,
                visited: HashSet::new(),
                pages: VecDeque::new(),
            }
        }

        // pub fn add_pages(&mut self, page: Page<Scrapable, U>) {
        //     self.pages.push_back(page);
        // }
    }

    // let mut ph = PageHandler::<BBCUrl>::new();

    let url = BBCUrl::parse("https://www.bbc.co.uk/news/articles/ceddenl8xz4o")?;
    println!("{:#?}", url);
    let page1: Page<ToScrape, BBCUrl> = Page::new_to_scrape(url);
    let page1: Box<Page<dyn Scrapable, BBCUrl>> = Box::new(page1);
    let page1: Page<WasScraped<BBCContent>, BBCUrl> = page1.scrape_in_place::<BBCContent>().await?;

    let url = BBCUrl::parse("https://www.bbc.co.uk/news/articles/c8009e2z4xlo")?;
    let page2 = Page::new_link_to(url, "Hello");
    let page2 = page2.scrape::<BBCContent>().await?;
    let page2 = Box::new(page2);

    let page3 = Box::new(
        Page::new_to_scrape(BBCUrl::parse(
            "https://www.bbc.co.uk/news/articles/cg66g0neweko",
        )?)
        .scrape::<BBCContent>()
        .await?,
    );

    let url = BBCUrl::parse("https://www.bbc.co.uk/news/articles/c0661dnmzezo")?;
    let page4: Box<Page<dyn Scrapable, BBCUrl>> = Box::new(Page::new_to_scrape(url));
    let page5: Box<Page<dyn Scrapable, BBCUrl>> = Box::new(Page::new_link_to(
        BBCUrl::parse("https://www.bbc.co.uk/news/articles/c6ppd6p12k4o")?,
        "Greens vow tax hike on wealthier to fund NHS and housing",
    ));
    let page6: Box<Page<dyn Scrapable, BBCUrl>> = Box::new(Page::new_to_scrape(BBCUrl::parse(
        "https://www.bbc.co.uk/news/articles/c9rrwe0ne7ro",
    )?));

    //Same as page 4
    let page4_2: Box<Page<dyn Scrapable, BBCUrl>> = Box::new(Page::new_to_scrape(BBCUrl::parse(
        "https://www.bbc.co.uk/news/articles/c0661dnmzezo",
    )?));
    let page5_2: Box<Page<dyn Scrapable, BBCUrl>> = Box::new(Page::new_link_to(
        BBCUrl::parse("https://www.bbc.co.uk/news/articles/c6ppd6p12k4o")?,
        "Greens vow tax hike on wealthier to fund NHS and housing",
    ));

    let mut visited: HashSet<Arc<BBCUrl>> = HashSet::new();

    visited.insert(page1.get_url_arc());
    visited.insert(page2.get_url_arc());
    visited.insert(page3.get_url_arc());
    //visited.insert(page4.get_url_arc());
    println!("Visited: {:#?}", visited);

    let mut pages: VecDeque<Box<Page<dyn Scrapable, BBCUrl>>> = VecDeque::new();

    pages.push_back(page4);
    pages.push_back(page5);
    pages.push_back(page6);
    //pages.push_back(page4_2);
    //pages.push_back(page5_2);

    println!("Pages: {:#?}", pages);

    let current_depth = 1;
    println!(
        " ------------------------------------- ITERATION {} -------------------------------------",
        current_depth
    );

    let pages_to_scrape = pages
        .drain(..)
        .collect::<Vec<Box<Page<dyn Scrapable, BBCUrl>>>>();

    println!("Pages to scrape: {:#?}", pages_to_scrape);

    let mut seen = HashSet::new();

    let mut unique_pages_to_scrape = pages_to_scrape
        .into_iter()
        .filter(|page| seen.insert(page.get_url_arc()))
        .collect::<Vec<Box<Page<dyn Scrapable, BBCUrl>>>>();

    println!("Unique Pages: {:#?}", unique_pages_to_scrape);

    unique_pages_to_scrape.retain(|page| {
        let url = page.get_url_arc();
        !visited.contains(&url)
    });
    println!(
        "Pages to scrape after retain: {:#?}",
        unique_pages_to_scrape
    );

    println!(
        "Pages to scrape this iteration: {:?}",
        unique_pages_to_scrape.len()
    );

    //? The Arc type is a reference-counted pointer that allows you to share ownership of a value across multiple threads. It is used to create a reference to the data that can be moved into the concurrent task.
    let visited_mutex = Arc::new(Mutex::new(visited));
    let pages_mutex = Arc::new(Mutex::new(pages));

    stream::iter(unique_pages_to_scrape.into_iter())
        .for_each_concurrent(None, |scrapable_page| {
            //? Each of the tasks need to have access to scraped_pages, but cant directly pass scraped_pages to them because it would mean multiple owners. What we are doing here is creating a new reference (Arc) to the data (.clone()). This new arc can then be moved into the concurrent task, giving it access to the shared data.
            let visited_mutex = visited_mutex.clone();
            let pages_mutex = pages_mutex.clone();

            //? here the async means creating an async block of code that can be awaited.
            //? The move means the closure takes ownership of the values it uses inside the closure (url, scraped_pages).
            async move {
                {
                    let mut visited_urls = visited_mutex.lock().await;
                    if visited_urls.contains(&scrapable_page.get_url_arc()) {
                        return;
                    }
                }

                if let Some(page) = scrapable_page.scrape_in_place::<BBCContent>().await.ok() {
                    let linked_pages = page
                        .get_all_page_links()
                        .into_iter()
                        .map(|page| Box::new(page) as Box<Page<dyn Scrapable, BBCUrl>>)
                        .collect::<Vec<Box<Page<dyn Scrapable, BBCUrl>>>>();

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
    let mut visited = visited_mutex.lock().await.clone();
    // let mut guard = visited_mutex.lock().await;
    // let mut visited = mem::replace(&mut *guard, HashSet::new());
    println!("Pages visited: {:#?}", visited);

    let mut guard = pages_mutex.lock().await;
    let mut pages = mem::replace(&mut *guard, VecDeque::new());

    println!("Pages left to scrape: {:#?}", pages);

    // Box::pin(self.get_pages_recursive_internal(max_depth, current_depth + 1)).await;

    println!("Finished!");

    Ok(())
}
