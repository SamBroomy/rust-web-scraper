
/// This is a macro that is used to create a new Url type.
/// # Example
///
/// Example of how to use the macro to generate the above boilerplate code
/// TODO: MACRO TO GENERATE ABOVE BOILERPLATE CODE (./sites/bbc/url.rs)
/// ```rust
/// use crate::create_url_type;
/// use crate::common::ScrapableContent;
///
///  // This is a placeholder type that will be used in the macro
/// #[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// struct ExampleContent;
///
/// // Custom parsing function for ExampleUrl
/// fn parse_example_url(url: &str) -> Result<String> {
///     todo!("Example of custom parsing function")
/// }
///
///
/// create_url_type!(
///     ExampleURL,
///     ExampleContent,
///     "https://www.example.com",
///     parse_example_url
/// );
///
/// impl ScrapableContent for ExampleContent {
///     type Url = ExampleURL;
///     fn from_scraped_page(url: &Self::Url, document: &Html) -> Result<Self> {
///         todo!("Ensure this function is implemented")
///     }
/// }

#[macro_export]
macro_rules! create_url_type {
    ($type_name:ident, $content_type:ty, $base_url:expr, $parse_function:expr) => {
        #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
        pub struct $type_name(String);

        impl AsRef<String> for $type_name {
            fn as_ref(&self) -> &String {
                &self.0
            }
        }

        impl TryFrom<String> for $type_name {
            type Error = Error;

            fn try_from(url: String) -> Result<Self> {
                match $parse_function(&url) {
                    Ok(parsed_url) => Ok($type_name(parsed_url)),
                    Err(e) => Err(e),
                }
            }
        }

        impl From<$type_name> for String {
            fn from(url: $type_name) -> String {
                url.0
            }
        }

        impl std::hash::Hash for $type_name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.as_ref().hash(state);
            }
        }

        impl UrlTrait for $type_name {
            fn base_url() -> &'static str {
                $base_url
            }

            fn to_string(&self) -> String {
                self.0.clone()
            }

            fn parse_url(url: &str) -> Result<String> {
                $parse_function(url)
            }
        }
    };
}
