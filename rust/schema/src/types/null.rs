use std::fmt;

use crate::prelude::*;

/// A null value
///
/// This is a struct, rather than a unit variant of `Primitive`, so that
/// it can be treated the same way as other variants when dispatching to
/// trait methods.
///
/// This is an empty struct, rather than a unit struct, because
/// Autosurgeon will not work with unit structs.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Null;

impl fmt::Display for Null {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "null")
    }
}

impl Serialize for Null {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_none()
    }
}

impl<'de> Deserialize<'de> for Null {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value.is_null() {
            true => Ok(Null {}),
            false => Err(serde::de::Error::custom("Expected a null value")),
        }
    }
}
