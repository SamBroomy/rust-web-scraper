use crate::Result;

use super::UrlTrait;

use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use reqwest::{Client, IntoUrl, Response};
use scraper::Html;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task;
use tokio::time::{sleep, Duration};
use tracing::instrument;
use tracing::{debug, info};

lazy_static! {
    static ref SEMAPHORE: Arc<Semaphore> = Arc::new(Semaphore::new(50)); // 10 permits
}

lazy_static! {
    static ref USER_AGENTS: Vec<&'static str> = vec![
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/83.0.4103.116 Safari/537.36",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:78.0) Gecko/20100101 Firefox/78.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_4) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/13.1 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.3 Safari/605.1.15",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:88.0) Gecko/20100101 Firefox/88.0",
    ];
}

lazy_static! {
    static ref CLIENT: Client = Client::builder()
        .timeout(Duration::from_secs(12))
        .connect_timeout(Duration::from_secs(10))
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::ACCEPT,
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
                    .parse()
                    .unwrap(),
            );
            headers.insert(
                reqwest::header::ACCEPT_LANGUAGE,
                "en-US,en;q=0.5".parse().unwrap(),
            );
            headers
        })
        .user_agent(*USER_AGENTS.choose(&mut rand::thread_rng()).unwrap())
        .pool_idle_timeout(Some(Duration::from_secs(10)))
        .pool_max_idle_per_host(10)
        .build()
        .unwrap();
}

/// Make a request to a given URL and return the scraper HTML content
#[instrument]
pub async fn make_request(url: &impl UrlTrait) -> Result<Html> {
    debug!("Making request to: {:?}", url.full_url());
    sleep(Duration::from_millis(50)).await; //? something pretty arbitrary but find it helps when sending off lots of requests at once
    let _permit = SEMAPHORE.acquire().await;
    let user_agent = USER_AGENTS.choose(&mut rand::thread_rng()).unwrap();
    let response = CLIENT
        .get(url.full_url())
        .header("User-Agent", *user_agent)
        .send()
        .await?;
    let html_content = response.text().await?;
    info!("---> Finished request to: {:?}", url.full_url());
    Ok(Html::parse_document(&html_content))
}
