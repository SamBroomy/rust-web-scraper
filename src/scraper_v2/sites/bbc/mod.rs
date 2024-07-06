pub mod error;
mod content;
mod scraper;
mod url;

pub use content::BBCContent;
pub use scraper::BBCScraper;
pub use url::{BBCRelatedTopicUrl, BBCNewsUrl};
