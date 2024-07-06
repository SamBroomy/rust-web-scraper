pub mod common;
mod error;
pub mod sites;

pub use self::error::{Error, Result};

use common::{MockDB, Page, Scrapable, Scraper, SurrealDb, ToScrape, UrlTrait, WasScraped, DB};
use sites::bbc::{BBCContent, BBCNewsUrl, BBCScraper};

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn setup_tracing() {
    let filter = EnvFilter::from_default_env()
        .add_directive("html5ever=error".parse().unwrap())
        .add_directive("selectors=error".parse().unwrap())
        .add_directive("debug".parse().unwrap());

    tracing_subscriber::registry()
        .with(fmt::layer().with_span_events(fmt::format::FmtSpan::CLOSE))
        .with(filter)
        .init();
}

pub async fn run() -> Result<()> {
    //let db = get_db("scraping").await?;
    setup_tracing();

    let url = BBCNewsUrl::parse("https://www.bbc.co.uk/news/articles/ceddenl8xz4o")?;
    println!("{:#?}", url);
    let page1 = Page::new_to_scrape(url);
    println!("{:#?}", page1);
    let page1 = page1.scrape::<BBCContent>().await?;
    println!("{:#?}", page1);

    let url = BBCNewsUrl::parse("https://www.bbc.co.uk/news/articles/ceddenl8xz4o")?;
    println!("{:#?}", url);
    let page1: Page<ToScrape, BBCNewsUrl> = Page::new_to_scrape(url);
    let page1: Box<Page<dyn Scrapable, BBCNewsUrl>> = Box::new(page1);
    let page1: Page<WasScraped<BBCContent>, BBCNewsUrl> =
        page1.scrape_in_place::<BBCContent>().await?;

    let url = BBCNewsUrl::parse("https://www.bbc.co.uk/news/articles/c8009e2z4xlo")?;
    let page2 = Page::new_link_to(url, "Hello");
    let page2 = page2.scrape::<BBCContent>().await?;
    let page2 = Box::new(page2);

    let page3 = Box::new(
        Page::new_to_scrape(BBCNewsUrl::parse(
            "https://www.bbc.co.uk/news/articles/cg66g0neweko",
        )?)
        .scrape::<BBCContent>()
        .await?,
    );

    let url = BBCNewsUrl::parse("https://www.bbc.co.uk/news/articles/c0661dnmzezo")?;
    let page4: Box<Page<dyn Scrapable, BBCNewsUrl>> = Box::new(Page::new_to_scrape(url));
    let page5: Box<Page<dyn Scrapable, BBCNewsUrl>> = Box::new(Page::new_link_to(
        BBCNewsUrl::parse("https://www.bbc.co.uk/news/articles/c6ppd6p12k4o")?,
        "Greens vow tax hike on wealthier to fund NHS and housing",
    ));
    let page6: Box<Page<dyn Scrapable, BBCNewsUrl>> = Box::new(Page::new_to_scrape(
        BBCNewsUrl::parse("https://www.bbc.co.uk/news/articles/c9rrwe0ne7ro")?,
    ));

    //Same as page 4
    let page4_2: Box<Page<dyn Scrapable, BBCNewsUrl>> = Box::new(Page::new_to_scrape(
        BBCNewsUrl::parse("https://www.bbc.co.uk/news/articles/c0661dnmzezo")?,
    ));
    let page5_2: Box<Page<dyn Scrapable, BBCNewsUrl>> = Box::new(Page::new_link_to(
        BBCNewsUrl::parse("https://www.bbc.co.uk/news/articles/c6ppd6p12k4o")?,
        "Greens vow tax hike on wealthier to fund NHS and housing",
    ));
    // let db = MockDB::<BBCContent>::default();
    // let db = Arc::new(Mutex::new(db));

    // let mut ph = PageHandler::<BBCUrl>::new();
    // ph.add_pages(vec![page4, page5, page6, page4_2, page5_2])
    //     .await;

    // let mut  ph2 = ph.clone();

    // ph.scrape_pages_recursive(db.clone(), 3).await;

    println!("_____________________ BBC Scraper ________________________");

    let db: DB<MockDB> = Arc::new(Mutex::new(MockDB::new()));

    let db: DB<SurrealDb> = Arc::new(Mutex::new(SurrealDb::new(None).await));

    let bbc_scraper = BBCScraper;

    let mut scraper = Scraper::new(bbc_scraper);

    // TODO: Get data from db and add already scraped pages to scraper

    scraper.initialize().await?;

    scraper
        .add_pages(vec![page4, page5, page6, page4_2, page5_2])
        .await;

    scraper
        .scrape_pages_recursive::<BBCContent, _>(&db, 10)
        .await;

    println!("Finished!");

    //println!("{:#?}", db.lock().await.get_content().lock().await);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    run().await?;

    Ok(())
}
