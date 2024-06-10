pub type Result<T> = std::result::Result<T, Error>;
//pub type Error = Box<dyn std::error::Error>;

use super::sites::error::*;

#[derive(Debug, derive_more::From)]
pub enum Error {
    #[from]
    Custom(String),
    ExampleError {
        too_high: u32,
        too_low: u32,
    },
    // -- Module
    //Common(super:common::Error),
    #[from]
    ScraperError(ScraperError),
    #[from]
    WikipediaError(WikipediaError),
    #[from]
    BBCError(BBCError),

    // -- Externals
    #[from]
    Io(std::io::Error),
    #[from]
    Reqwest(reqwest::Error),
    #[from]
    Anyhow(anyhow::Error),
}

// For custom error messages.
impl Error {
    pub fn custom(val: impl std::fmt::Display) -> Self {
        Self::Custom(val.to_string())
    }
}

impl From<&str> for Error {
    fn from(val: &str) -> Self {
        Self::Custom(val.to_string())
    }
}

// Error boilerplate.
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
