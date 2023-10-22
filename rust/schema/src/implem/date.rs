use std::str::FromStr;

use common::{once_cell::sync::Lazy, regex::Regex};

use crate::{prelude::*, Date};

impl Date {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        (
            elem("date", [("iso-8601-date", &self.value)], &self.value),
            Losses::none(),
        )
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
