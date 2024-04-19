pub type Result<T> = std::result::Result<T, Error>;
use derive_more::From;
use reqwest;
#[derive(Debug, From)]
pub enum Error {
    NoArticleFound,
    NoTitleFound,
    NoContentFound,
    NoRelatedTopicsFound,
    InvalidRelatedLink,

    //Wikipedia specific
    ShortDescriptionNotFound,
    TableNotFound,
    TableTitleNotFount,
    NoCategoriesFound,

    NoUrlsFound,
    #[from]
    Reqwest(reqwest::Error),
    #[from]
    Io(std::io::Error),
    #[from]
    AnyHow(anyhow::Error),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}
