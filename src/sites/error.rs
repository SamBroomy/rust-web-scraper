pub use super::bbc::error::*;
pub use super::wikipedia::error::*;

#[derive(Debug, derive_more::From)]
pub enum ScraperError {
    #[from]
    ExampleError { too_high: u32, too_low: u32 },
}
// Error boilerplate.
impl core::fmt::Display for ScraperError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for ScraperError {}
