// #![allow(unused)] // For beginning only.
pub mod get_db;
// pub mod scraper;
pub mod scraper_v2;

use scraper_v2::*;

use futures::stream::futures_unordered::IntoIter;
use lazy_static::lazy_static;
use reqwest;
use reqwest::Url;
use scraper::pages::bbc::{Article, LinkedArticle, URL};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::fmt::format;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::io::Write;
use std::path::Path;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tokio;

use crate::scraper::pages;

async fn setup() -> Result<()> {
    let init_links_file = Path::new("init_links.json");

    if !init_links_file.exists() {
        println!("Creating init_links.json file!");
        let init_links = scraper::pages::bbc::scrape_base_url().await?;
        let init_links_json = json!(init_links);
        let mut file = File::create(init_links_file)?;
        serde_json::to_writer_pretty(&mut file, &init_links_json)?;
    }
    let init_links: HashSet<URL> = serde_json::from_reader(File::open(init_links_file)?)?;
    println!("{:?}", init_links);

    let linked_articles_file = Path::new("linked_articles.json");

    if !linked_articles_file.exists() {
        println!("Creating linked_articles.json file!");
        let linked_articles = scrape_urls(&init_links).await;

        let linked_articles_json = json!(linked_articles);
        let mut file = File::create(linked_articles_file)?;
        serde_json::to_writer_pretty(&mut file, &linked_articles_json)?;
    }
    let linked_articles: Vec<LinkedArticle> =
        serde_json::from_reader(File::open(linked_articles_file)?)?;

    Ok(())
}

async fn scrape_urls(urls: &HashSet<URL>) -> Vec<LinkedArticle> {
    let results = futures::future::join_all(
        urls.iter()
            .map(|url| async { scraper::pages::bbc::scrape_page(&url.clone()).await }),
    )
    .await;
    results
        .into_iter()
        .filter_map(|result| result.ok())
        .collect::<Vec<LinkedArticle>>()
}

async fn insert_data(
    db: &Surreal<Client>,
    linked_articles: &Vec<LinkedArticle>,
    visited: &mut HashSet<URL>,
    articles_to_fetch: &mut HashSet<URL>,
) -> Result<()> {
    println!("Inserting data into database!");
    // This could be turned into a function.
    for linked_article in linked_articles.iter() {
        // Destructure linked_article
        let LinkedArticle { article, links } = linked_article;
        let url = article.url.clone();

        // Add article to database
        let created: Option<Article> = db
            .update(("articles", url.url()))
            .content(article.clone())
            .await?;

        // Add article to visited
        visited.insert(url.clone());
        // Add links to database
        let sql = format!(
            "RELATE articles:`{}`->links->{:?};",
            url.url(),
            links
                .into_iter()
                .map(|link| format!("articles:`{}`", link.url()))
                .collect::<Vec<String>>()
        );
        db.query(sql).await?;

        // Add links to fetch
        articles_to_fetch.extend(links.clone());
    }
    println!("Data inserted into database!");

    Ok(())
}

async fn get_pages_recursive_internal(
    max_depth: u32,
    current_depth: u32,
    db: &Surreal<Client>,
    articles_to_fetch: &mut HashSet<URL>,
    visited: &mut HashSet<URL>,
) -> Result<()> {
    if current_depth > max_depth {
        return Ok(());
    }

    println!(
        " ------------------------------------- ITERATION {} -------------------------------------",
        current_depth
    );
    println!("Number of Visited pages: {:?}", visited.len());

    articles_to_fetch.retain(|url| !visited.contains(url));
    println!(
        "Articles to fetch for this iteration: {:?}",
        articles_to_fetch.len()
    );
    visited.extend(articles_to_fetch.clone());

    let linked_articles = scrape_urls(articles_to_fetch).await;
    println!("Linked articles fetched: {:?}", linked_articles.len());
    insert_data(&db, &linked_articles, visited, articles_to_fetch).await?;

    Box::pin(get_pages_recursive_internal(
        max_depth,
        current_depth + 1,
        db,
        articles_to_fetch,
        visited,
    ))
    .await;

    Ok(())
}

async fn get_pages_recursive(
    mut max_depth: u32,
    db: &Surreal<Client>,
    articles_to_fetch: &mut HashSet<URL>,
    visited: &mut HashSet<URL>,
) -> Result<()> {
    if max_depth > 10 {
        println!("Max depth is too high, setting to 10!");
        max_depth = 10;
    }
    let max_depth = max_depth;
    get_pages_recursive_internal(max_depth, 0, db, articles_to_fetch, visited).await
}

#[tokio::main]
async fn main() -> Result<()> {
    let db = Surreal::new::<Ws>("127.0.0.1:8080").await?;
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;

    db.use_ns("scraping").use_db("bbc_news").await?;
    println!("Connected to database!");
    let sql_query_path = Path::new("./sql/01-setup.surql");
    let sql_query = std::fs::read_to_string(sql_query_path)?;
    println!("Executing SQL query: {:?}", sql_query);

    db.query(sql_query).await?;

    setup().await?;
    let mut articles_to_fetch = scraper::pages::bbc::scrape_base_url().await?;
    let mut visited = HashSet::<URL>::new();

    get_pages_recursive(3, &db, &mut articles_to_fetch, &mut visited).await?;

    Ok(())
}
