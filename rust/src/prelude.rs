pub use defaults::Defaults;
pub use serde::{Deserialize, Serialize};
pub use serde_json::Value;
pub use serde_with::skip_serializing_none;
pub use std::{
    collections::BTreeMap,
    convert::AsRef,
    fmt::{self, Display},
    sync::Arc,
};
pub use strum::AsRefStr;

/// A null value
///
/// This is a struct, rather than a unit variant of `Primitive`, so that
/// it can be treated the same way as other variants when dispatching to
/// trait methods.
#[derive(Clone, Debug)]
pub struct Null {}

impl Display for Null {
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

/// A boolean value
pub type Boolean = bool;

/// An integer value
///
/// Uses `i64` for maximum precision.
pub type Integer = i64;

/// A floating point value (a.k.a real number)
///
/// Uses `i64` for maximum precision.
pub type Number = f64;

/// An array value (a.k.a. vector)
pub type Array = Vec<Primitive>;

/// An object value (a.k.a map, dictionary)
///
/// Uses `BTreeMap` to preserve order.
pub type Object = BTreeMap<String, Primitive>;

/// The set of primitive (non-Entity) node types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Primitive {
    Null(Null),
    Boolean(Boolean),
    Integer(Integer),
    Number(Number),
    String(String),
    Object(Object),
    Array(Array),
}
