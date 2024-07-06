use crate::Result;

use serde::Serialize;
use std::fmt::Debug;
use std::hash::Hash;

/// This is a trait that is used to represent a url.
pub trait UrlTrait:
    Hash + Debug + TryFrom<String> + AsRef<String> + Eq + Send + Sync + Clone + Serialize
{
    /// To create a new Url type from a string. Can also be used on the type itself.
    fn parse(url: impl Into<String>) -> std::result::Result<Self, Self::Error>
    where
        // So the sized bit basically means that the type has a known size at compile time. Eg the new method cant be called on the trait itself, it has to be called on a type that implements the trait.
        Self: Sized,
    {
        // Check if the url is already a Self type and return it.

        Self::try_from(url.into())
    }

    fn initialise() -> Self;

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
