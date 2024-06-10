pub mod bbc;
pub mod error;
pub mod wikipedia;

use crate::common::UrlTrait;
use crate::Result;

pub use error::*;

use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::Surreal;

use async_trait::async_trait;

#[async_trait]
pub trait SiteScraper {
    fn new(db: &Surreal<Client>) -> Self;
    fn add_links(&mut self, url: impl UrlTrait);
    async fn get_pages_recursive(&mut self, depth: usize) -> Result<()>;
}
