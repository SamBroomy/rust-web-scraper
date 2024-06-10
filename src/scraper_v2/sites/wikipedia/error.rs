#[derive(Debug, derive_more::From)]
pub enum WikipediaError {
    #[from]
    Custom(String),
    ExampleError {
        too_high: u32,
        too_low: u32,
    },
    ParseError {
        reason: String,
    },
    InvalidUrl {
        url: String,
        reason: String,
    },
}

// Error boilerplate.
impl core::fmt::Display for WikipediaError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for WikipediaError {}
