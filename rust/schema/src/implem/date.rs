use std::str::FromStr;

use common::{
    chrono::{self, Datelike},
    eyre::{Report, Result},
    inflector::Inflector,
    once_cell::sync::Lazy,
    regex::Regex,
};

use crate::{prelude::*, Date};

impl Date {
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
        use common::chrono::Local;
        use interim::{parse_date_string, Dialect};

        // If matches an ISO 8601 then use that...
        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^\d{4}(-\d\d(-\d\d)?)?$").expect("Unable to create regex"));
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
        use common::chrono::Local;
        use interim::{parse_date_string, Dialect};

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
