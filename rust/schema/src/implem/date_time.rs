use std::{str::FromStr, sync::LazyLock};

use chrono::Utc;
use eyre::{Report, Result};
use inflector::Inflector;
use interim::{Dialect, parse_date_string};
use regex::Regex;

use crate::{DateTime, prelude::*};

impl DateTime {
    /// Encode a date-time as a DOM HTML attribute
    pub fn to_dom_attr(name: &str, date_time: &Self, context: &mut DomEncodeContext) {
        context.push_attr(&name.to_kebab_case(), &date_time.value.to_string());
    }

    pub fn to_jats_special(&self) -> (String, Losses) {
        use stencila_codec_jats_trait::encode::elem;

        (
            elem(
                "date-time",
                [("iso-8601-date-time", &self.value)],
                &self.value,
            ),
            Losses::none(),
        )
    }
}

impl TryFrom<&DateTime> for chrono::DateTime<Utc> {
    type Error = Report;

    fn try_from(date_time: &DateTime) -> Result<Self, Self::Error> {
        let date_time = parse_date_string(&date_time.value, Utc::now(), Dialect::Us)?;
        Ok(date_time)
    }
}

impl TryFrom<&DateTime> for time::OffsetDateTime {
    type Error = Report;

    fn try_from(date_time: &DateTime) -> Result<Self, Self::Error> {
        let date_time: chrono::DateTime<Utc> = date_time.try_into()?;

        let date_time = time::OffsetDateTime::from_unix_timestamp(date_time.timestamp())?;
        Ok(date_time)
    }
}

impl FromStr for DateTime {
    type Err = ErrReport;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        // If matches an ISO 8601 partial or complete date/date-time then use that directly.
        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(
                r"^(\d{4}(-\d\d(-\d\d)?)?|\d\d:\d\d(:\d\d(\.\d+)?)?|\d{4}-\d\d-\d\dT\d\d:\d\d(:\d\d(\.\d+)?)?(Z|[+-]\d\d:\d\d)?)$",
            )
            .expect("Unable to create regex")
        });
        if REGEX.is_match(string) {
            return Ok(Self::new(string.to_string()));
        }

        // Otherwise parse as date-time, falling back to just using the string.
        let string = if let Ok(date_time) = parse_date_string(string, Utc::now(), Dialect::Us) {
            let repr = date_time.to_rfc3339();
            // Remove redundant timestamp when dates such as "22 Feb 2022" are parsed
            repr.trim_end_matches("T00:00:00+00:00").into()
        } else {
            string.to_string()
        };

        Ok(Self::new(string))
    }
}
