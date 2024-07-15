/// This is a macro that is used to create a new Url type.
/// # Example
///
/// Example of how to use the macro to generate the above boilerplate code
/// TODO: MACRO TO GENERATE ABOVE BOILERPLATE CODE (./sites/bbc/url.rs)
/// ```rust
/// define_url_type!(MyCustomUrl, "https://example.com", my_custom_parse_function);
/// define_content_type!(MyCustomContent, MyCustomUrl, MyCustomRelatedUrl);
/// define_error_type!(MyCustomError,
///     NotFound { url: String },
///     ParseError { reason: String }
/// );
/// define_scraper!(MyCustomScraper, MyCustomUrl, MyCustomContent);
/// ```
#[macro_export]
macro_rules! define_url_type {
    ($name:ident, $base_url:expr, $parse_fn:expr, $initial_url:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct $name(String);

        impl AsRef<String> for $name {
            fn as_ref(&self) -> &String {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = $crate::Error;

            fn try_from(url: String) -> Result<Self> {
                Self::parse_url(&url).map($name)
            }
        }

        impl From<$name> for String {
            fn from(url: $name) -> String {
                url.0
            }
        }

        impl Hash for $name {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.as_ref().hash(state);
            }
        }

        impl UrlTrait for $name {
            fn base_url() -> &'static str {
                $base_url
            }

            fn to_string(&self) -> String {
                self.0.clone()
            }

            #[instrument(skip_all, level = "trace", err(level = "trace"), fields(stripped_url))]
            fn parse_url(url: &str) -> Result<String> {
                $parse_fn(url)
            }

            fn initialise() -> Self {
                Self($initial_url.to_string())
            }
        }
    };
}

#[macro_export]
macro_rules! define_content_type {
    ($name:ident, $url_type:ty, $related_url_type:ty) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct $name {
            title: String,
            content: Vec<String>,
            url: $url_type,
            related_pages: HashSet<Page<LinkTo, $url_type>>,
            related_topics: HashSet<RelatedPage<$related_url_type>>,
        }

        impl ScrapableContent for $name {
            type Url = $url_type;
            type RelatedUrl = $related_url_type;

            fn get_title(&self) -> String {
                self.title.clone()
            }

            fn get_url(&self) -> Self::Url {
                self.url.clone()
            }

            fn get_related_pages(&self) -> HashSet<Page<LinkTo, Self::Url>> {
                self.related_pages.clone()
            }

            fn get_related_topics(&self) -> HashSet<RelatedPage<Self::RelatedUrl>> {
                self.related_topics.clone()
            }

            // Note: The from_scraped_page method still needs to be implemented manually
            // as it's likely to be site-specific
        }
    };
}

#[macro_export]
macro_rules! define_error_type {
    ($name:ident, $($variant:ident { $($field:ident: $ty:ty),* }),*) => {

        pub enum $name {
            $($variant { $($field: $ty),* }),*
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }

        impl std::error::Error for $name {}
    };
}
