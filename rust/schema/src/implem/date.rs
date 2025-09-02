use std::{str::FromStr, sync::LazyLock};

use chrono::{self, Datelike};
use eyre::{Report, Result};
use inflector::Inflector;
use regex::Regex;

use crate::{Date, prelude::*};

impl Date {
    /// Get the year part of a date
    pub fn year(&self) -> Option<u32> {
        self.value
            .split('-')
            .next()
            .and_then(|year| year.parse().ok())
    }

    /// Get the month part of a date
    pub fn month(&self) -> Option<u32> {
        self.value
            .split('-')
            .nth(1)
            .and_then(|month| month.parse().ok())
    }

    /// Get the day part of a date
    pub fn day(&self) -> Option<u32> {
        self.value
            .split('-')
            .nth(2)
            .and_then(|day| day.parse().ok())
    }

    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        (
            elem("date", [("iso-8601-date", &self.value)], &self.value),
            Losses::none(),
        )
    }

    /// Encode a date as a DOM HTML attribute
    pub fn to_dom_attr(name: &str, date: &Self, context: &mut DomEncodeContext) {
        context.push_attr(&name.to_kebab_case(), &date.value.to_string());
    }
}

impl FromStr for Date {
    type Err = ErrReport;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        use chrono::Local;
        use interim::{Dialect, parse_date_string};

        // If matches an ISO 8601 then use that...
        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"^\d{4}(-\d\d(-\d\d)?)?$").expect("Unable to create regex")
        });
        if REGEX.is_match(string) {
            return Ok(Self::new(string.to_string()));
        }

        // Otherwise parse as date, falling back to just using the string
        let string = if let Ok(date_time) = parse_date_string(string, Local::now(), Dialect::Us) {
            date_time.date_naive().to_string()
        } else {
            string.to_string()
        };

        Ok(Self::new(string))
    }
}

impl TryFrom<&Date> for chrono::NaiveDate {
    type Error = Report;

    fn try_from(date: &Date) -> Result<Self, Self::Error> {
        use chrono::Local;
        use interim::{Dialect, parse_date_string};

        let date_time = parse_date_string(&date.value, Local::now(), Dialect::Us)?;
        let date = date_time.date_naive();

        Ok(date)
    }
}

impl TryFrom<&Date> for time::Date {
    type Error = Report;

    fn try_from(date: &Date) -> Result<Self, Self::Error> {
        let date: chrono::NaiveDate = date.try_into()?;

        let year = date.year();
        let month = time::Month::try_from(date.month() as u8)?;
        let day = date.day() as u8;
        let date = time::Date::from_calendar_date(year, month, day)?;

        Ok(date)
    }
}
