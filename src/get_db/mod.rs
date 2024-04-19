use anyhow::Result;
use std::fs;
use std::path::Path;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

pub async fn get_db(db_name: impl Into<String>) -> Result<Surreal<Client>> {
    let db_client = Surreal::new::<Ws>("127.0.0.1:8080").await?;
    db_client
        .signin(Root {
            username: "root",
            password: "root",
        })
        .await?;

    db_client.use_ns("scraping").use_db(db_name.into()).await?;

    let sql_file = Path::new("../../sql/01-setup.surql");

    if let Ok(definition) = fs::read_to_string(sql_file) {
        db_client.query(definition).await?;
    }

    Ok(db_client)
}
