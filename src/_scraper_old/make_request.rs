use super::url::Url;
use anyhow::Result;
use lazy_static::lazy_static;
use reqwest;
use scraper::Html;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task;
use tokio::time::{sleep, Duration};
lazy_static! {
    static ref SEMAPHORE: Arc<Semaphore> = Arc::new(Semaphore::new(50)); // 10 permits
}
lazy_static!(
    static ref CLIENT: reqwest::Client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10))
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3"
        )
        .pool_idle_timeout(Some(Duration::from_secs(10)))
        .pool_max_idle_per_host(10)
        .build()
        .unwrap();
);

pub async fn make_request(url: &Url) -> Result<Html> {
    sleep(Duration::from_millis(50)).await; // sleep for 100 milliseconds
    let _permit = SEMAPHORE.acquire().await;

    let response = CLIENT.get(url.url()).send().await?;
    let html_content = response.text().await?;
    println!("---> Finished request to: {:?}", url.url());
    Ok(Html::parse_document(&html_content))
}
