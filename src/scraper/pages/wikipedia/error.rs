pub type Result<T> = anyhow::Result<T, Error>;
use derive_more::From;
use reqwest;
#[derive(Debug, From)]
pub enum Error {
    NoPageContentFound,
    NoTitleFound,
    NoContentFound,
    UnableToExtractContent,
    NoCategoriesFound,

    #[from]
    Reqwest(reqwest::Error),

    #[from]
    AnyHow(anyhow::Error),
}
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}
