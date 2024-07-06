use recursive_scraper::{run, Result};

#[tokio::main]
async fn main() -> Result<()> {
    run().await?;
    Ok(())
}
