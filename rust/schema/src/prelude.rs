use chrono::{DateTime, Utc};
pub use derivative::Derivative;
use float_cmp::approx_eq;
pub use serde::{Deserialize, Serialize};
pub use serde_json::Value;
pub use serde_with::skip_serializing_none;
pub use std::{
    collections::BTreeMap,
    convert::AsRef,
    fmt::{self, Display},
    sync::Arc,
};
pub use strum::{AsRefStr, EnumString};

use crate::Primitive;

/// A null value
///
/// This is a struct, rather than a unit variant of `Primitive`, so that
/// it can be treated the same way as other variants when dispatching to
/// trait methods.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
/// Uses `f64` for maximum precision.
///
/// Needs to be a newtype so that we can implement `PartialEq` which is
/// not implemented for f64.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Number(pub f64);

impl Number {
    pub fn new(num: f64) -> Self {
        Self(num)
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        approx_eq!(f64, self.0, other.0, ulps = 2)
    }
}

impl Eq for Number {}

impl std::hash::Hash for Number {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // See caveats to this approach: https://stackoverflow.com/a/39647997
        self.0.to_bits().hash(state)
    }
}

impl std::str::FromStr for Number {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Number(f64::from_str(s)?))
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for Number {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// An array value (a.k.a. vector)
pub type Array = Vec<Primitive>;

/// An object value (a.k.a map, dictionary)
///
/// Uses `BTreeMap` to preserve order.
pub type Object = BTreeMap<String, Primitive>;

/// A newtype derived from `String`
///
/// Defined primarily so that a customized `Patchable` implementation
/// can be defined for strings where it is more appropriate to replace,
/// rather than diff the string.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cord(pub String);

// Convenience functions for `Date`

impl From<DateTime<Utc>> for crate::Date {
    fn from(date_time: DateTime<Utc>) -> Self {
        Self {
            value: date_time.to_rfc3339(),
            ..Default::default()
        }
    }
}

impl From<String> for crate::Date {
    fn from(string: String) -> Self {
        Self {
            value: string,
            ..Default::default()
        }
    }
}

impl crate::Date {
    pub fn now() -> Self {
        Self {
            value: Utc::now().to_rfc3339(),
            ..Default::default()
        }
    }
}
