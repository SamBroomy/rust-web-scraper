use recursive_scraper::run;
use recursive_scraper::Result;

#[tokio::main]
async fn main() -> Result<()> {
    run().await?;

    Ok(())
}
