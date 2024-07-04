use crate::Result;

use crate::common::{ScrapableContent, UrlTrait};

use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::instrument;

#[async_trait]
pub trait DatabaseService<C: ScrapableContent>: Send + Sync + Debug {
    async fn save_content(&self, content: &C) -> Result<()>;
    // Additional methods for link management or querying could be added here
}

#[derive(Debug, Clone)]
pub struct MockDB<C: ScrapableContent> {
    // This is a mock database that stores content in memory.
    content: Arc<Mutex<HashMap<String, C>>>,
}
#[async_trait]
impl<C: ScrapableContent> DatabaseService<C> for MockDB<C> {
    #[instrument(skip_all, name = "MockDB::save_content", level = "debug", fields(url = %c.get_url().to_string()))]
    async fn save_content(&self, c: &C) -> Result<()> {
        {
            let mut content = self.content.lock().await;
            content.insert(c.get_url().to_string(), c.clone());
        }
        Ok(())
    }
}

impl<C> MockDB<C>
where
    C: ScrapableContent,
{
    fn new() -> Self {
        Self {
            content: Arc::new(Mutex::new(HashMap::<String, C>::new())),
        }
    }
}

impl<C> Default for MockDB<C>
where
    C: ScrapableContent,
{
    fn default() -> Self {
        Self::new()
    }
}

pub type DB<C> = Arc<Mutex<dyn DatabaseService<C>>>;
