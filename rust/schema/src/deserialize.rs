//! Functions used in type definitions for specifying how properties can be deserialized

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

#[derive(Debug, Deserialize)]
#[serde(untagged, crate = "common::serde")]
enum StringOrObject<T> {
    String(String),
    Object(T),
}

#[derive(Debug, Deserialize)]
#[serde(untagged, crate = "common::serde")]
enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

/// Deserialize an object from a string using the [`FromStr`] trait
fn from_str<'de, T, D>(string: &str) -> Result<T, D::Error>
where
    T: FromStr,
    D: Deserializer<'de>,
{
    T::from_str(string).map_err(|_| {
        Error::custom(format!(
            "error parsing string as `{}`",
            std::any::type_name::<T>()
        ))
    })
}

/// Split a string by a separator and deserialize each item using [`FromStr`]
fn split_by<'de, T, D>(string: String, sep: &str) -> Result<Vec<T>, D::Error>
where
    T: DeserializeOwned + FromStr,
    D: Deserializer<'de>,
{
    string
        .split(sep)
        .map(|item| from_str::<T, D>(item.trim()))
        .collect()
}

/// Split a string by whitespace and deserialize each item using [`FromStr`]
fn split_whitespace<'de, T, D>(string: String) -> Result<Vec<T>, D::Error>
where
    T: DeserializeOwned + FromStr,
    D: Deserializer<'de>,
{
    string.split_whitespace().map(from_str::<T, D>).collect()
}

/// Deserialize a vector from a string of comma separated values or from an array
pub fn option_csv_or_array<'de, T, D>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
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
pub fn option_ssv_or_array<'de, T, D>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
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

/// Deserialize a vector from one or many of the items
pub fn one_or_many<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: DeserializeOwned,
    D: Deserializer<'de>,
{
    Ok(match OneOrMany::<T>::deserialize(deserializer)? {
        OneOrMany::One(one) => vec![one],
        OneOrMany::Many(many) => many,
    })
}

/// Deserialize an optional vector from one or many of the items
pub fn option_one_or_many<'de, T, D>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
where
    T: DeserializeOwned,
    D: Deserializer<'de>,
{
    Ok(match Option::<OneOrMany<T>>::deserialize(deserializer)? {
        Some(OneOrMany::One(one)) => Some(vec![one]),
        Some(OneOrMany::Many(many)) => Some(many),
        None => None,
    })
}

/// Deserialize a struct from a string or object
#[allow(unused)]
pub fn string_or_object<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: DeserializeOwned + FromStr,
    D: Deserializer<'de>,
{
    Ok(match StringOrObject::<T>::deserialize(deserializer)? {
        StringOrObject::String(string) => from_str::<T, D>(&string)?,
        StringOrObject::Object(object) => object,
    })
}

/// Deserialize an optional struct from a string or object
pub fn option_string_or_object<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: DeserializeOwned + FromStr,
    D: Deserializer<'de>,
{
    Ok(
        match Option::<StringOrObject<T>>::deserialize(deserializer)? {
            Some(StringOrObject::String(string)) => Some(from_str::<T, D>(&string)?),
            Some(StringOrObject::Object(object)) => Some(object),
            None => None,
        },
    )
}

/// Deserialize a vector from one or many, strings or objects
pub fn option_one_or_many_string_or_object<'de, T, D>(
    deserializer: D,
) -> Result<Option<Vec<T>>, D::Error>
where
    T: DeserializeOwned + FromStr,
    D: Deserializer<'de>,
{
    Ok(
        match Option::<OneOrMany<StringOrObject<T>>>::deserialize(deserializer)? {
            Some(OneOrMany::One(one)) => Some(match one {
                StringOrObject::String(string) => vec![from_str::<T, D>(&string)?],
                StringOrObject::Object(object) => vec![object],
            }),
            Some(OneOrMany::Many(many)) => Some(
                many.into_iter()
                    .map(|string_or_object| match string_or_object {
                        StringOrObject::String(string) => from_str::<T, D>(&string),
                        StringOrObject::Object(object) => Ok(object),
                    })
                    .collect::<Result<Vec<T>, _>>()?,
            ),
            None => None,
        },
    )
}
