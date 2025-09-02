use chrono::Utc;
use eyre::{Report, Result};
use interim::{Dialect, parse_date_string};

use crate::{DateTime, prelude::*};

impl DateTime {
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
