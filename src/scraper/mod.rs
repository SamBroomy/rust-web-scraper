mod error;
pub mod make_request;
pub mod pages;
mod url;
use crate::insert_data;
use crate::scraper::pages::ScrapedPage;
use anyhow::{Ok, Result};
use futures::stream::futures_unordered::IntoIter;
use futures::stream::{self, StreamExt};
use itertools::Itertools;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::Surreal;

use tokio::sync::Mutex as AsyncMutex;

pub use url::{BBCUrl, Url, UrlTrait, WikipediaUrl};

use self::pages::WikipediaPage;

pub struct Scraper<'a> {
    db: &'a Surreal<Client>,
    visited: HashSet<Url>, // To keep track of visited urls. // Could just use a HashSet to track visited and to visited
    to_visit: VecDeque<Url>, // To perform breadth-first search use a VecDeque
}

impl<'a> Scraper<'a> {
    pub fn new(db: &'a Surreal<Client>) -> Self {
        Scraper {
            db,
            visited: HashSet::new(),
            to_visit: VecDeque::new(),
        }
    }
    pub fn add_links(&mut self, url: &Url) {
        self.to_visit.push_back(url.clone())
    }

    async fn insert_data_to_db(&self, url: &WikipediaUrl, page: &WikipediaPage) -> Result<()> {
        let created: Option<WikipediaPage> = self
            .db
            .update(("articles", url.specific_path()))
            .content(page)
            .await?;

        println!("Created page: {:#}", url.specific_path());
        self.insert_links_to_db(url, page.get_all_page_links())
            .await?;

        Ok(())
    }

    async fn insert_links_to_db(
        &self,
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
            self.db.query(sql).await?;
        }
        Ok(())
    }

    async fn scrape_pages(&self, pages_to_scrape: Vec<Url>) -> Vec<ScrapedPage> {
        let mut scraped_pages = Arc::new(AsyncMutex::new(Vec::new()));

        stream::iter(pages_to_scrape.into_iter())
            .for_each_concurrent(None, |url| {
                let scraped_pages = scraped_pages.clone();
                async move {
                    match url {
                        Url::Wikipedia(url) => {
                            if let Some(scraped_page) =
                                WikipediaPage::get_page_with_content(&url).await.ok()
                            {
                                self.insert_data_to_db(&url, &scraped_page).await;
                                let mut pages = scraped_pages.lock().await;
                                pages.push(ScrapedPage::Wikipedia(scraped_page))
                            }
                        }
                        Url::BBC(url) => (),
                    }
                }
            })
            .await;

        let final_pages = scraped_pages.lock().await;
        final_pages.clone()

        // stream::iter(pages_to_scrape.into_iter())
        //     .then(|url| async move {
        //         match url {
        //             Url::Wikipedia(url) => {
        //                 if let Some(scraped_page) =
        //                     WikipediaPage::get_page_without_content(&url).await.ok()
        //                 {
        //                     self.insert_data_to_db(&url, &scraped_page).await;
        //                     Some(ScrapedPage::Wikipedia(scraped_page))
        //                 } else {
        //                     None
        //                 }
        //             }
        //             Url::BBC(url) => None,
        //         }
        //     })
        //     .filter_map(futures::future::ready)
        //     .collect::<Vec<ScrapedPage>>()
        //     .await
    }

    async fn get_pages_recursive_internal(
        &mut self,
        max_depth: u32,
        current_depth: u32,
    ) -> Result<()> {
        if current_depth > max_depth {
            return Ok(());
        }

        println!(
        " ------------------------------------- ITERATION {} -------------------------------------",
        current_depth);
        if self.to_visit.is_empty() {
            println!("No more pages to visit!");
            return Ok(());
        } else {
            println!("Number of Visited pages: {:?}", self.to_visit.len());
        }

        let mut pages_to_scrape = self.to_visit.drain(..).collect::<HashSet<Url>>();
        pages_to_scrape.retain(|url| !self.visited.contains(url));

        println!(
            "Pages to scrape this iteration: {:?}",
            pages_to_scrape.len()
        );
        let scraped_pages = self
            .scrape_pages(pages_to_scrape.into_iter().collect())
            .await;

        self.to_visit.extend(
            scraped_pages
                .iter()
                .map(|page| page.get_urls())
                .flatten()
                .filter(|url| !self.visited.contains(url)),
        );

        Box::pin(self.get_pages_recursive_internal(max_depth, current_depth + 1)).await;

        Ok(())
    }

    pub async fn get_pages_recursive(&mut self, mut max_depth: u32) -> Result<()> {
        if max_depth > 10 {
            println!("Max depth is too high, setting to 10!");
            max_depth = 10;
        }
        let max_depth = max_depth;
        self.get_pages_recursive_internal(max_depth, 0).await
    }
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
//         let sql = format!(
//             "RELATE articles:`{}`->links->{:?};",
//             url.url(),
//             links
//                 .into_iter()
//                 .map(|link| format!("articles:`{}`", link.url()))
//                 .collect::<Vec<String>>()
//         );
//         db.query(sql).await?;

//         // Add links to fetch
//         articles_to_fetch.extend(links.clone());
//     }
//     println!("Data inserted into database!");

//     Ok(())
// }
