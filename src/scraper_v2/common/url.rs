use crate::Result;

use scraper::Html;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

/// This is a trait that is used to represent a url.
pub trait UrlTrait: Hash + Debug + TryFrom<String> + AsRef<String> {
    type ContentType: FromScrapedPage<Self>;

    /// To create a new Url type from a string. Can also be used on the type itself.
    fn new(url: impl Into<String>) -> std::result::Result<Self, Self::Error>
    where
        // So the sized bit basically means that the type has a known size at compile time. Eg the new method cant be called on the trait itself, it has to be called on a type that implements the trait.
        Self: Sized,
    {
        // Check if the url is already a Self type and return it.

        Self::try_from(url.into())
    }

    /// The base url for the site.
    fn base_url() -> &'static str;
    /// Returns the url as a string.
    fn to_string(&self) -> String;
    /// Returns the full url.
    fn full_url(&self) -> String {
        format!("{}{}", Self::base_url(), self.to_string())
    }
    /// This is a helper method that takes a url and returns a parsed url.
    fn parse_url(url: &str) -> Result<String>;

    /// This is a helper method that takes a collection of urls and returns a collection of Self types.
    fn from_collection<T, U, V>(urls: U) -> V
    where
        Self: Sized,
        T: Into<String>,
        U: IntoIterator<Item = T>,
        V: FromIterator<Self>,
    {
        urls.into_iter()
            .filter_map(|s| Self::try_from(s.into()).ok())
            .collect()
    }
}

/// This is a trait that is used to convert a scraped page into a type.
pub trait FromScrapedPage<U: UrlTrait>: Debug {
    /// This is a helper method that takes a url and a document and returns a Result of the type.
    fn from_scraped_page(url: &U, document: &Html) -> Result<Self>
    where
        Self: Sized;
}

/// TODO: Get this working <br>
/// This is a macro that is used to create a new Url type.
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
            type ContentType = $content_type;

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

// Example of how to use the macro to generate the above boilerplate code
// TODO: MACRO TO GENERATE ABOVE BOILERPLATE CODE (./sites/bbc/url.rs)
// use crate::common::FromScrapedPage;
// use crate::create_url_type;
// use scraper::Html;
// #[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
// struct BBCSportContent; // This is a placeholder type that will be used in the macro

// /// Custom parsing function for BBCSportUrl
// fn parse_bbc_sport_url(url: &str) -> Result<String> {
//     todo!("Example of custom parsing function")
// }

// create_url_type!(
//     BBCSportURL,
//     BBCSportContent,
//     "https://www.bbc.co.uk/",
//     parse_bbc_sport_url
// );
// /* ! WARNING ! For this to work ContentType in the url trait must be:
// trait UrlTrait {
//     type ContentType
//     NOT
//     type ContentType: FromScrapedPage<Self>;
//     as the macro will not be able to generate the code for the latter
// }
// haven't quite worked out how to make this work with the macro yet as it requires a type that is not yet defined
// */
// impl FromScrapedPage<BBCSportURL> for BBCSportContent {
//     fn from_scraped_page(url: &BBCSportURL, document: &Html) -> Result<Self> {
//         todo!("Ensure this function is implemented")
//     }
// }
