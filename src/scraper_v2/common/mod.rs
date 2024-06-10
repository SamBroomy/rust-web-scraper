mod make_request;
mod page;
mod scraper;
mod url;

pub use make_request::make_request;
pub use page::{LinkTo, Page, PageState, ScrapableContent, Scraped, ToScrape};
pub use url::UrlTrait;