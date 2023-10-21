use std::str::FromStr;

use common::serde::{
    de::{DeserializeOwned, Error},
    Deserialize, Deserializer,
};

#[derive(Debug, Deserialize)]
#[serde(untagged, crate = "common::serde")]
enum StringOrArray<T> {
    String(String),
    Array(Vec<T>),
}

/// Split a string by a separator and deserialize each item using [`FromStr`]
fn split_by<'de, T, D>(string: String, sep: &str) -> Result<Vec<T>, D::Error>
where
    T: DeserializeOwned + FromStr,
    D: Deserializer<'de>,
{
    string
        .split(sep)
        .map(|item| T::from_str(item.trim()))
        .collect::<Result<Vec<T>, _>>()
        .map_err(|_| {
            Error::custom(format!(
                "error parsing string as `{}`",
                std::any::type_name::<T>()
            ))
        })
}

/// Split a string by whitespace and deserialize each item using [`FromStr`]
fn split_whitespace<'de, T, D>(string: String) -> Result<Vec<T>, D::Error>
where
    T: DeserializeOwned + FromStr,
    D: Deserializer<'de>,
{
    string
        .split_whitespace()
        .map(|item| T::from_str(item))
        .collect::<Result<Vec<T>, _>>()
        .map_err(|_| {
            Error::custom(format!(
                "error parsing string as `{}`",
                std::any::type_name::<T>()
            ))
        })
}

/// Deserialize a vector from a string of comma separated values or from an array
pub fn optional_csv_or_array<'de, T, D>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
where
    T: DeserializeOwned + FromStr,
    D: Deserializer<'de>,
{
    Ok(
        match Option::<StringOrArray<T>>::deserialize(deserializer)? {
            Some(StringOrArray::<T>::String(string)) => Some(split_by::<T, D>(string, ",")?),
            Some(StringOrArray::<T>::Array(array)) => Some(array),
            None => None,
        },
    )
}

/// Deserialize a vector from a string of whitespace separated values or from an array
pub fn optional_ssv_or_array<'de, T, D>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
where
    T: DeserializeOwned + FromStr,
    D: Deserializer<'de>,
{
    Ok(
        match Option::<StringOrArray<T>>::deserialize(deserializer)? {
            Some(StringOrArray::<T>::String(string)) => Some(split_whitespace::<T, D>(string)?),
            Some(StringOrArray::<T>::Array(array)) => Some(array),
            None => None,
        },
    )
}
