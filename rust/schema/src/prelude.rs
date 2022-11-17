pub use derivative::Derivative;
use float_cmp::approx_eq;
use serde::ser::Error;
pub use serde::{Deserialize, Serialize};
pub use serde_json::Value;
pub use serde_with::skip_serializing_none;
pub use smartstring::{LazyCompact, SmartString};
pub use std::{
    collections::BTreeMap,
    convert::AsRef,
    fmt::{self, Display},
    sync::Arc,
};
pub use strum::{AsRefStr, EnumString};

use crate::{Date, DateTime, Duration, Primitive, Time, TimeUnit, Timestamp};

pub type Suid = SmartString<LazyCompact>;

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

impl Default for Number {
    fn default() -> Self {
        Self(0f64)
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
        // Use JSON stringify here because that deals with ensuring decimal point for
        // floats that are close to integers.
        let json = serde_json::to_string(&self.0).expect("Should always be able to stringify");
        write!(f, "{}", json)
    }
}

impl std::ops::Deref for Number {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Custom deserialization for primitives serialized with an internal `type` tag

macro_rules! deserialize_date_time {
    ($type:ty) => {
        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                if let Ok(value) = serde_json::Value::deserialize(deserializer) {
                    if let Some(object) = value.as_object() {
                        if let Some(stringify!($type)) =
                            object.get("type").and_then(|value| value.as_str())
                        {
                            return Ok(Self {
                                value: object
                                    .get("value")
                                    .and_then(|value| value.as_str())
                                    .unwrap_or_default()
                                    .to_string(),
                                ..Default::default()
                            });
                        }
                    }
                }
                Err(serde::de::Error::custom("Not a type"))
            }
        }
    };
}

deserialize_date_time!(Date);
deserialize_date_time!(Time);
deserialize_date_time!(DateTime);

macro_rules! deserialize_time_united {
    ($type:ty) => {
        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use std::str::FromStr;
                if let Ok(value) = serde_json::Value::deserialize(deserializer) {
                    if let Some(object) = value.as_object() {
                        if let Some(stringify!($type)) =
                            object.get("type").and_then(|value| value.as_str())
                        {
                            return Ok(Self {
                                value: object
                                    .get("value")
                                    .and_then(|value| value.as_i64())
                                    .unwrap_or_default(),
                                time_unit: object
                                    .get("timeUnit")
                                    .and_then(|value| value.as_str())
                                    .and_then(|value| TimeUnit::from_str(value).ok())
                                    .unwrap_or(TimeUnit::Microsecond),
                                ..Default::default()
                            });
                        }
                    }
                }
                Err(serde::de::Error::custom("Not a type"))
            }
        }
    };
}

deserialize_time_united!(Timestamp);
deserialize_time_united!(Duration);

// Convenience methods for `Date` etc

const DATE_FORMAT: &str = "%Y-%m-%d";
const TIME_FORMAT: &str = "%H:%M:%S";
const DATE_TIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

impl Date {
    /// Convert a date to a string parsable by SQL databases
    ///
    /// This could be improved on a lot!
    pub fn to_sql(&self) -> String {
        let colons = self.value.matches(':').count();
        match colons {
            0 => [&self.value, ":00:00"].concat(),
            1 => [&self.value, ":00"].concat(),
            _ => self.value.clone(),
        }
    }
}

impl From<String> for Date {
    fn from(string: String) -> Self {
        Self {
            value: string,
            ..Default::default()
        }
    }
}

impl ToString for Date {
    fn to_string(&self) -> String {
        self.value.to_owned()
    }
}

impl From<chrono::NaiveDate> for Date {
    fn from(date: chrono::NaiveDate) -> Self {
        Self {
            value: date.format(DATE_FORMAT).to_string(),
            ..Default::default()
        }
    }
}

impl From<chrono::DateTime<chrono::Utc>> for Date {
    fn from(date: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            value: date.format(DATE_FORMAT).to_string(),
            ..Default::default()
        }
    }
}

impl Time {
    /// Convert a time to a string parsable by SQL databases
    ///
    /// See note for `Date::to_sql`.
    pub fn to_sql(&self) -> String {
        let colons = self.value.matches(':').count();
        match colons {
            0 => [&self.value, ":00:00"].concat(),
            1 => [&self.value, ":00"].concat(),
            _ => self.value.clone(),
        }
    }
}

impl From<String> for Time {
    fn from(string: String) -> Self {
        Self {
            value: string,
            ..Default::default()
        }
    }
}

impl ToString for Time {
    fn to_string(&self) -> String {
        self.value.to_owned()
    }
}

impl From<chrono::NaiveTime> for Time {
    fn from(date: chrono::NaiveTime) -> Self {
        Self {
            value: date.format(TIME_FORMAT).to_string(),
            ..Default::default()
        }
    }
}

impl DateTime {
    // Get the `DateTime` now
    pub fn now() -> Self {
        Self::from(chrono::Utc::now())
    }

    /// Convert a datetime to a string parsable by SQL databases
    ///
    /// See note for `Date::to_sql`.
    pub fn to_sql(&self) -> String {
        let colons = self.value.matches(':').count();
        match colons {
            0 => [&self.value, ":00:00"].concat(),
            1 => [&self.value, ":00"].concat(),
            _ => self.value.clone(),
        }
    }
}

impl From<String> for DateTime {
    fn from(string: String) -> Self {
        Self {
            value: string,
            ..Default::default()
        }
    }
}

impl ToString for DateTime {
    fn to_string(&self) -> String {
        self.value.to_owned()
    }
}

impl From<chrono::NaiveDateTime> for DateTime {
    fn from(date: chrono::NaiveDateTime) -> Self {
        Self {
            value: date.format(DATE_TIME_FORMAT).to_string(),
            ..Default::default()
        }
    }
}

impl From<chrono::DateTime<chrono::Utc>> for DateTime {
    fn from(date_time: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            value: date_time.to_rfc3339(),
            ..Default::default()
        }
    }
}

impl Timestamp {
    // Get the `Timestamp` now
    pub fn now() -> Self {
        Self::from(chrono::Utc::now())
    }

    /// Convert a timestamp to a `chrono::NaiveDateTime`
    pub fn to_chrono_datetime(
        &self,
    ) -> std::result::Result<chrono::NaiveDateTime, std::fmt::Error> {
        use TimeUnit::*;
        let epoch = chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd(1970, 1, 1),
            chrono::NaiveTime::from_hms(0, 0, 0),
        );
        let duration = match self.time_unit {
            Day => chrono::Duration::days(self.value),
            Hour => chrono::Duration::hours(self.value),
            Minute => chrono::Duration::minutes(self.value),
            Second => chrono::Duration::seconds(self.value),
            Millisecond => chrono::Duration::milliseconds(self.value),
            Microsecond => chrono::Duration::microseconds(self.value),
            Nanosecond => chrono::Duration::nanoseconds(self.value),
            _ => {
                return Err(std::fmt::Error::custom(&format!(
                    "Unable to convert a timestamp with unit `{}` to a `chrono::NaiveDateTime`",
                    self.time_unit.as_ref()
                )))
            }
        };

        match epoch.checked_add_signed(duration) {
            Some(date) => Ok(date),
            None => Err(std::fmt::Error::custom("")),
        }
    }

    /// Convert a timestamp to an ISO 8601 string
    pub fn to_iso8601(&self) -> std::result::Result<String, std::fmt::Error> {
        Ok(self
            .to_chrono_datetime()?
            .format(DATE_TIME_FORMAT)
            .to_string())
    }

    /// Convert a timestamp to a `DateTime`
    pub fn to_date_time(&self) -> std::result::Result<DateTime, std::fmt::Error> {
        Ok(DateTime::from(self.to_chrono_datetime()?))
    }

    /// Convert a date to a string parseable by SQL databases
    pub fn to_sql(&self) -> std::result::Result<String, std::fmt::Error> {
        self.to_iso8601()
    }
}

impl From<chrono::DateTime<chrono::Utc>> for Timestamp {
    fn from(date_time: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            value: date_time.timestamp_millis(),
            time_unit: TimeUnit::Millisecond,
            ..Default::default()
        }
    }
}

impl Duration {
    /// Create a duration from the number of minutes
    pub fn from_mins(value: i64) -> Self {
        Self {
            value,
            time_unit: TimeUnit::Minute,
            ..Default::default()
        }
    }

    /// Create a duration from the number of seconds
    pub fn from_secs(value: i64) -> Self {
        Self {
            value,
            time_unit: TimeUnit::Second,
            ..Default::default()
        }
    }

    /// Create a duration from the number of milliseconds
    pub fn from_millis(value: i64) -> Self {
        Self {
            value,
            time_unit: TimeUnit::Millisecond,
            ..Default::default()
        }
    }

    /// Create a duration from the number of microseconds
    pub fn from_micros(value: i64) -> Self {
        Self {
            value,
            time_unit: TimeUnit::Microsecond,
            ..Default::default()
        }
    }

    /// Create a string representation of the duration using the units
    /// that make most sense to a human
    ///
    /// A more sophisticated approach to this would be to convert to
    /// a `chrono::Duration` and use the `chrono_humanize` crate.
    pub fn humanize(&self) -> String {
        use TimeUnit::*;

        let value = self.value as f64;
        let (value, time_unit) = match self.time_unit {
            Minute => {
                if value > 1.44e3 {
                    (value / 1.44e3, &Day)
                } else if value > 6e1 {
                    (value / 6e1, &Hour)
                } else {
                    (value, &self.time_unit)
                }
            }
            Second => {
                if value > 8.64e4 {
                    (value / 8.64e4, &Day)
                } else if value > 3.6e3 {
                    (value / 3.6e3, &Hour)
                } else if value > 6e1 {
                    (value / 6e1, &Minute)
                } else {
                    (value, &self.time_unit)
                }
            }
            Millisecond => {
                if value > 8.64e7 {
                    (value / 8.64e7, &Day)
                } else if value > 3.6e6 {
                    (value / 3.6e6, &Hour)
                } else if value > 6e4 {
                    (value / 6e4, &Minute)
                } else if value > 1e3 {
                    (value / 1e3, &Second)
                } else {
                    (value, &self.time_unit)
                }
            }
            Microsecond => {
                if value > 3.6e9 {
                    (value / 3.6e9, &Hour)
                } else if value > 6e7 {
                    (value / 6e7, &Minute)
                } else if value > 1e6 {
                    (value / 1e6, &Second)
                } else if value > 1e3 {
                    (value / 1e3, &Millisecond)
                } else {
                    (value, &self.time_unit)
                }
            }
            _ => (value as f64, &self.time_unit),
        };

        format!("{:.3}{}", value, time_unit.to_si())
    }

    /// Convert a duration to a string parsable by SQL databases such as Postgres, SQLite, DuckDB etc
    pub fn to_sql(&self) -> String {
        [&self.value.to_string(), self.time_unit.as_ref()].concat()
    }
}

impl ToString for Duration {
    fn to_string(&self) -> String {
        [&self.value.to_string(), " ", &self.time_unit.to_si()].concat()
    }
}

impl TimeUnit {
    pub fn to_si(&self) -> String {
        use TimeUnit::*;
        match self {
            Year => "yr",
            Month => "mo",
            Week => "wk",
            Day => "d",
            Hour => "h",
            Minute => "min",
            Second => "s",
            Millisecond => "ms",
            Microsecond => "µs",
            Nanosecond => "ns",
            Picosecond => "ps",
            Femtosecond => "fs",
            Attosecond => "as",
        }
        .to_string()
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
