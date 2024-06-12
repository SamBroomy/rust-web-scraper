use core::hash;
use std::collections::{HashSet, VecDeque};
use std::rc::Rc;
use std::sync::Arc;

use my_crate::get_db::get_db;
use my_crate::scraper_v2::common::{
    make_request, LinkTo, Page, PageState, Scrapable, Scraped, ToScrape, UrlTrait, WasScraped,
};
use my_crate::scraper_v2::sites::bbc::{BBCContent, BBCUrl};
use my_crate::scraper_v2::Result;

use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

#[tokio::main]
async fn main() -> Result<()> {
    //let db = get_db("scraping").await?;

    let url = BBCUrl::new("https://www.bbc.co.uk/news/articles/ceddenl8xz4o")?;
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

    let url = BBCUrl::new("https://www.bbc.co.uk/news/articles/ceddenl8xz4o")?;
    println!("{:#?}", url);
    let page1: Page<ToScrape, BBCUrl> = Page::new_to_scrape(url);
    let page1_url: std::sync::Arc<BBCUrl> = page1.get_url_arc();
    let page1: Box<Page<dyn Scrapable, BBCUrl>> = Box::new(page1);
    let page1_url_1 = page1.as_ref().get_url_arc();
    // let page1_transition = page1.as_ref().scrape::<BBCContent>().await?;

    let url = BBCUrl::new("https://www.bbc.co.uk/news/articles/c8009e2z4xlo")?;
    let page2 = Page::new_link_to(url, "Hello");
    let page2_url: Arc<BBCUrl> = page2.get_url_arc();
    let page2: Box<Page<dyn Scrapable, BBCUrl>> = Box::new(page2);

    let page3: Box<Page<dyn Scraped, BBCUrl>> = Box::new(
        Page::new_to_scrape(BBCUrl::new("https://www.bbc.co.uk/news/articles/")?)
            .scrape::<BBCContent>()
            .await?,
    );
    let mut visited: HashSet<Arc<BBCUrl>> = HashSet::new();

    visited.insert(page1_url);
    visited.insert(page2_url);

    // let mut pages = VecDeque::new();
    let mut pages: VecDeque<Box<Page<dyn Scrapable, BBCUrl>>> = VecDeque::new();

    pages.push_back(page1);
    pages.push_back(page2);

    let page1 = Rc::new(BBCUrl::new(
        "https://www.bbc.co.uk/news/articles/ceddenl8xz4o",
    )?);
    let page2 = Rc::new(BBCUrl::new(
        "https://www.bbc.co.uk/news/articles/c8009e2z4xlo",
    )?);

    visited.insert(Rc::clone(&page1));
    visited.insert(Rc::clone(&page1));
    visited.insert(page2);

    let a = 124 as i32;

    let b = 1235 as i32;

    //visited.insert(page2);

    println!("{:#?}", visited);
    // pages.push_back(Page::new_to_scrape(page1).as_ref());
    // pages.push_back(Page::new_link_to(page2, "Title").as_ref());

    // ph.visited.insert(page1);

    let current_depth = 1;

    println!(
        " ------------------------------------- ITERATION {} -------------------------------------",
        current_depth
    );

    println!("Pages: {:#?}", pages);

    // let mut pages_to_scrape = pages
    //     .drain(..)
    //     .collect::<HashSet<Box<Page<dyn Scrapable, BBCUrl>>>>();

    // println!("Pages to scrape: {:#?}", pages_to_scrape);
    // pages_to_scrape.retain(|page| {
    //     let url: BBCUrl = page.as_ref();
    //     !visited.contains(url)
    // });

    // pages_to_scrape
    //     .into_iter()
    //     .for_each(|page: Box<Page<dyn Scrapable, BBCUrl>>| {
    //         let url: BBCUrl = page.as_ref();

    //         visited.into_iter().for_each(|visited_url: Rc<BBCUrl>| {
    //             if visited_url == url {
    //                 println!("Page: {:#?}", page);
    //             }
    //         });
    //     });
    // pages_to_scrape.retain(|url| !self.visited.contains(url.as_ref()));

    // println!(
    //     "Pages to scrape this iteration: {:?}",
    //     pages_to_scrape.len()
    // );
    // let scraped_pages = self
    //     .scrape_pages(pages_to_scrape.into_iter().collect())
    //     .await;

    // self.pages.extend(
    //     scraped_pages
    //         .iter()
    //         .map(|page| page.get_urls())
    //         .flatten()
    //         .filter(|url| !self.visited.contains(url)),
    // );

    ////////////////////////////////////////////////////////////////////////////////////////

    // let url = BBCUrl::new("https://www.bbc.co.uk/news/world-europe-55231203")?;

    // println!("{:#?}", url);

    // let page3 = Page::new_to_scrape(url).scrape().await?;

    // println!("{:#?}", page3);

    // let output = make_request(page1.as_ref()).await?;

    // println!("{:#?}", output);

    // let a = url.as_ref();

    // let url =
    //     WikipediaUrl::new("https://en.wikipedia.org/wiki/Rust_(programming_language)").unwrap();
    // let page = WikipediaPage::get_page_with_content(&url).await?;

    // let mut my_scraper = Scraper::new(&db);

    // my_scraper.add_links(&url.into());

    // my_scraper.get_pages_recursive(2).await;

    println!("Finished!");

    Ok(())
}
