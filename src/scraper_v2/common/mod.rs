mod content;
mod macros;
mod make_request;
mod model;
mod page;
mod scraper;
mod url;

pub use make_request::make_request;
pub use page::{
    LinkTo, Page, PageHandler, PageState, RelatedPage, Scrapable, Scraped, ToScrape, WasScraped,
};
pub use url::UrlTrait;

pub use scraper::{Scraper, SiteSpecificScraper};

pub use content::ScrapableContent;

pub use model::{DatabaseService, MockDB, SurrealDb, DB};
