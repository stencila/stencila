use codec::{
    common::{
        indexmap::IndexMap,
        serde::{Deserialize, Serialize},
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
/// See https://citeproc-js.readthedocs.io/en/latest/csl-json/markup.html#date-fields
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, crate = "codec::common::serde")]
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

    /// Legacy format for backwards compatibility
    DateParts {
        #[serde(rename = "date-parts")]
        date_parts: Vec<Vec<i32>>,
    },
}

/// Additional metadata for dates
///
/// Contains optional metadata that can accompany CSL date fields,
/// including temporal context, precision indicators, and repository-specific information.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(crate = "codec::common::serde")]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", crate = "codec::common::serde")]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

/// Represents approximate date information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, crate = "codec::common::serde")]
pub enum Circa {
    Bool(bool),
    Year(i64),
    Text(String),
}

/// Core date components
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "codec::common::serde")]
pub struct DateParts {
    pub year: i64,
    pub month: Option<u8>,
    pub day: Option<u8>,
}

/// Convert CSL date field to a Stencila Date
pub fn convert_csl_date(csl_date: &DateField) -> Option<Date> {
    match csl_date {
        DateField::DateParts { date_parts }
        | DateField::Single { date_parts, .. }
        | DateField::Range { date_parts, .. } => {
            if let Some(parts) = date_parts.first() {
                let year = parts.first().map(|y| y.to_string()).unwrap_or_default();
                let month = parts.get(1).map(|m| format!("-{m:02}")).unwrap_or_default();
                let day = parts.get(2).map(|d| format!("-{d:02}")).unwrap_or_default();
                Some(Date::new(format!("{year}{month}{day}")))
            } else {
                None
            }
        }
        DateField::Literal { literal, .. } => Some(Date::new(literal.to_string())),
        DateField::Raw { raw, .. } => Some(Date::new(raw.to_string())),
        DateField::Edtf { edtf, .. } => Some(Date::new(edtf.to_string())),
    }
}
