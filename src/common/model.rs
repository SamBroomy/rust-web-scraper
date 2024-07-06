use crate::Result;

use crate::common::{RelatedPage, ScrapableContent, UrlTrait};

use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use std::sync::Arc;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tokio::fs;
use tokio::sync::Mutex;
use tracing::instrument;

#[async_trait]
pub trait DatabaseService: Send + Sync + Debug {
    async fn save_content<C: ScrapableContent + 'static>(&self, content: &C) -> Result<()>;
    // Additional methods for link management or querying could be added here
}

/// This is a mock database that stores content in memory.
#[derive(Debug, Clone, Default)]
pub struct MockDB {
    // This is a mock database that stores content in memory.
    content: Arc<Mutex<HashMap<String, Box<dyn Any + Send>>>>,
}
impl MockDB {
    pub fn new() -> Self {
        Self {
            content: Arc::new(Mutex::new(HashMap::<String, Box<dyn Any + Send>>::new())),
        }
    }

    pub fn get_content(&self) -> Arc<Mutex<HashMap<String, Box<dyn Any + Send>>>> {
        Arc::clone(&self.content)
    }
}
#[async_trait]
impl DatabaseService for MockDB {
    #[instrument(skip_all, name = "MockDB::save_content", level = "debug", fields(url = %c.get_url().to_string()))]
    async fn save_content<C: ScrapableContent + 'static>(&self, c: &C) -> Result<()> {
        {
            let mut content = self.content.lock().await;
            content.insert(c.get_url().to_string(), Box::new(c.clone()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SurrealDb {
    db: Arc<Mutex<Surreal<Client>>>,
}

impl SurrealDb {
    pub async fn new(db_name: impl Into<Option<String>>) -> Self {
        Self {
            db: Arc::new(Mutex::new(Self::get_client(db_name).await.unwrap())),
        }
    }

    async fn get_client(db_name: impl Into<Option<String>>) -> Result<Surreal<Client>> {
        let db_client = Surreal::new::<Ws>("127.0.0.1:8080").await?;
        db_client
            .signin(Root {
                username: "root",
                password: "root",
            })
            .await?;

        let db_name = db_name.into().unwrap_or("test".to_string());

        db_client.use_ns("scraping").use_db(db_name).await?;

        let sql_file = Path::new("../../../sql/01-setup.surql");

        if let Ok(definition) = fs::read_to_string(sql_file).await {
            db_client.query(definition).await?;
        }

        Ok(db_client)
    }
}

#[async_trait]
impl DatabaseService for SurrealDb {
    #[instrument(skip_all, name = "SurrealDb::save_content", level = "debug", fields(url = %content.get_url().to_string()), err(level = "warn"))]
    async fn save_content<C: ScrapableContent + 'static>(&self, content: &C) -> Result<()> {
        {
            let _: Option<C> = self
                .db
                .lock()
                .await
                .update(("pages", content.get_url().to_string()))
                .content(content.clone())
                .await?;
        }

        let sql = format!(
            "RELATE pages:`{}`->links_to->{:?};",
            content.get_url().to_string(),
            content
                .get_related_pages()
                .into_iter()
                .map(|page| format!("pages:`{}`", page.get_url_arc().to_string()))
                .collect::<Vec<String>>()
        );
        {
            self.db.lock().await.query(sql).await?;
        }

        let topics = content.get_related_topics();

        stream::iter(topics.into_iter())
            .for_each_concurrent(None, |topic| {
                let db = self.db.clone();

                async move {
                    let _: Option<RelatedPage<C::RelatedUrl>> = db
                        .lock()
                        .await
                        .update(("topics", topic.get_title()))
                        .content(topic.clone())
                        .await
                        .unwrap();
                }
            })
            .await;

        let topic_names = content
            .get_related_topics()
            .into_iter()
            .map(|topic| format!("topics:`{}`", topic.get_title().to_string()))
            .collect::<Vec<String>>();

        let sql = format!(
            "RELATE pages:`{}`->has_topic->{:?};",
            content.get_url().to_string(),
            topic_names
        );

        self.db.lock().await.query(sql).await?;

        Ok(())
    }
}

pub type DB<DatabaseService> = Arc<Mutex<DatabaseService>>;
