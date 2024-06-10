#[derive(Debug, derive_more::From)]
pub enum BBCError {
    #[from]
    Custom(String),
    ExampleError {
        too_high: u32,
        too_low: u32,
    },

    InvalidUrl {
        url: String,
        reason: String,
    },

    NoArticleFound {
        url: String,
    },
    NoTitleFound {
        url: String,
    },
    NoContentFound {
        url: String,
    },
    NoRelatedTopicsFound {
        url: String,
    },
}

// Error boilerplate.
impl core::fmt::Display for BBCError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for BBCError {}
