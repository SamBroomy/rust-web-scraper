use core::hash;
use std::collections::{HashSet, VecDeque};
use std::mem;
use std::rc::Rc;
use std::sync::Arc;

use my_crate::get_db::get_db;
use my_crate::scraper_v2::common::{
    make_request, LinkTo, Page, PageHandler, PageScraper, PageState, Scrapable, Scraped, ToScrape,
    UrlTrait, WasScraped,
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

    let mut ph = PageHandler::<BBCUrl>::new();
    ph.add_pages(vec![page4, page5, page6, page4_2, page5_2])
        .await;

    ph.scrape_pages_recursive::<BBCContent>(3).await;

    println!("Page Handler: {:#?}", ph);

    println!("Finished!");

    Ok(())
}
