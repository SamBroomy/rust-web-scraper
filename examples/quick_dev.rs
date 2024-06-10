use my_crate::get_db::get_db;
use my_crate::scraper_v2::common::{make_request, Page, UrlTrait};
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

    let url = BBCUrl::new("https://www.bbc.co.uk/news/articles/c8009e2z4xlo")?;

    println!("{:#?}", url);

    let page2 = Page::new_to_scrape(url).scrape::<BBCContent>().await?;

    println!("{:#?}", page2);

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
