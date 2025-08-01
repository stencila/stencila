use std::str::FromStr;

use serde::Deserialize;

use codec::{
    common::{
        eyre::{Report, bail},
        indexmap::IndexMap,
        serde_json::Value,
        serde_with::skip_serializing_none,
    },
    schema::Date,
};

/// A CSL date field
///
/// Represents dates in CSL-JSON format with support for various date representations
/// including structured date parts, literal strings, raw formats, and EDTF.
///
/// See:
/// - https://docs.citationstyles.org/en/stable/specification.html#appendix-iv-variables (Date Variables)
/// - https://citeproc-js.readthedocs.io/en/latest/csl-json/markup.html#date-fields
#[derive(Deserialize)]
#[serde(untagged)]
pub enum DateField {
    /// Date with parts (year, month, day) and optional metadata
    Single {
        #[serde(rename = "date-parts")]
        date_parts: Vec<Vec<i32>>,

        #[serde(flatten)]
        meta: DateMeta,
    },

    /// Date range with start and end parts
    Range {
        #[serde(rename = "date-parts")]
        date_parts: Vec<Vec<i32>>,

        #[serde(flatten)]
        meta: DateMeta,
    },

    /// Literal date string
    Literal {
        literal: String,

        #[serde(flatten)]
        meta: DateMeta,
    },

    /// Raw date string
    Raw {
        raw: String,

        #[serde(flatten)]
        meta: DateMeta,
    },

    /// Extended Date/Time Format
    Edtf {
        edtf: String,

        #[serde(flatten)]
        meta: DateMeta,
    },
}

/// Additional metadata for dates
///
/// Contains optional metadata that can accompany CSL date fields,
/// including temporal context, precision indicators, and repository-specific information.
#[skip_serializing_none]
#[derive(Deserialize)]
pub struct DateMeta {
    /// Season information
    pub season: Option<Season>,

    /// Approximate date information
    pub circa: Option<Circa>,

    /// Literal date representation
    pub literal: Option<String>,

    /// ISO 8601 date-time string (used by repositories like CrossRef)
    #[serde(rename = "date-time")]
    pub date_time: Option<String>,

    /// Unix timestamp in milliseconds
    pub timestamp: Option<i64>,

    /// Version information for database records
    pub version: Option<String>,

    /// Additional date-related fields
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

/// Represents seasonal information
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

/// Represents approximate date information
#[derive(Deserialize)]
#[serde(untagged)]
pub enum Circa {
    Bool(bool),
    Year(i64),
    Text(String),
}

impl TryFrom<DateField> for Date {
    type Error = Report;

    fn try_from(value: DateField) -> Result<Self, Self::Error> {
        Ok(match value {
            DateField::Single { date_parts, .. } | DateField::Range { date_parts, .. } => {
                if let Some(parts) = date_parts.first() {
                    let year = parts.first().map(|y| y.to_string()).unwrap_or_default();
                    let month = parts.get(1).map(|m| format!("-{m:02}")).unwrap_or_default();
                    let day = parts.get(2).map(|d| format!("-{d:02}")).unwrap_or_default();
                    Date::new(format!("{year}{month}{day}"))
                } else {
                    bail!("No date parts")
                }
            }
            DateField::Literal { literal, .. } => Date::from_str(&literal)?,
            DateField::Raw { raw, .. } => Date::from_str(&raw)?,
            DateField::Edtf { edtf, .. } => Date::from_str(&edtf)?,
        })
    }
}
