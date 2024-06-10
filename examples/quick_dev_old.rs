#![allow(unused)] // For beginning only.
use anyhow::{Ok, Result};
use futures::stream::futures_unordered::IntoIter;
use itertools::Itertools;
use lazy_regex::{regex, regex_replace_all};
use lazy_static::lazy_static;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use my_crate::get_db::get_db;
use my_crate::scraper::make_request::make_request;
use my_crate::scraper::pages::WikipediaPage;
use my_crate::scraper::Scraper;
use my_crate::scraper::{Url, UrlTrait, WikipediaUrl};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::fmt::format;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tokio;
use tokio::task::LocalEnterGuard;

async fn insert_data_to_db(
    db: &Surreal<Client>,
    url: &WikipediaUrl,
    page: &WikipediaPage,
) -> Result<()> {
    let created: Option<WikipediaPage> = db
        .update(("articles", url.specific_path()))
        .content(page)
        .await?;

    println!("{:#?}", created);
    insert_links_to_db(db, url, page.get_all_page_links()).await?;

    Ok(())
}

async fn insert_links_to_db(
    db: &Surreal<Client>,
    url: &WikipediaUrl,
    page_links: HashSet<WikipediaUrl>,
) -> Result<()> {
    for chunk in &page_links.into_iter().chunks(100) {
        let sql = format!(
            "RELATE articles:`{}`->links->{:?};",
            url.specific_path(),
            chunk
                .into_iter()
                .map(|link| format!("articles:`{}`", link.specific_path()))
                .collect::<Vec<String>>()
        );
        println!("{}", sql);
        db.query(sql).await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let db = get_db("scraping").await?;

    let url =
        WikipediaUrl::new("https://en.wikipedia.org/wiki/Rust_(programming_language)").unwrap();
    let page = WikipediaPage::get_page_with_content(&url).await?;

    let mut my_scraper = Scraper::new(&db);

    my_scraper.add_links(&url.into());

    my_scraper.get_pages_recursive(2).await;

    println!("Finished!");

    Ok(())
}
// async fn insert_data(
//     db: &Surreal<Client>,
//     linked_articles: &Vec<LinkedArticle>,
//     visited: &mut HashSet<URL>,
//     articles_to_fetch: &mut HashSet<URL>,
// ) -> Result<()> {
//     println!("Inserting data into database!");
//     // This could be turned into a function.
//     for linked_article in linked_articles.iter() {
//         // Destructure linked_article
//         let LinkedArticle { article, links } = linked_article;
//         let url = article.url.clone();

//         // Add article to database
//         let created: Option<Article> = db
//             .update(("articles", url.url()))
//             .content(article.clone())
//             .await?;

//         // Add article to visited
//         visited.insert(url.clone());
//         // Add links to database
// let sql = format!(
//     "RELATE articles:`{}`->links->{:?};",
//     url.url(),
//     links
//         .into_iter()
//         .map(|link| format!("articles:`{}`", link.url()))
//         .collect::<Vec<String>>()
// );
//         db.query(sql).await?;

//         // Add links to fetch
//         articles_to_fetch.extend(links.clone());
//     }
//     println!("Data inserted into database!");

//     Ok(())
// }
