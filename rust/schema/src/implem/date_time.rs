use common::{
    chrono,
    eyre::{Report, Result},
};

use crate::{DateTime, prelude::*};

impl DateTime {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

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

impl TryFrom<&DateTime> for chrono::DateTime<chrono::Utc> {
    type Error = Report;

    fn try_from(date_time: &DateTime) -> Result<Self, Self::Error> {
        use common::chrono::Utc;
        use interim::{Dialect, parse_date_string};

        let date_time = parse_date_string(&date_time.value, Utc::now(), Dialect::Us)?;
        Ok(date_time)
    }
}

impl TryFrom<&DateTime> for time::OffsetDateTime {
    type Error = Report;

    fn try_from(date_time: &DateTime) -> Result<Self, Self::Error> {
        let date_time: chrono::DateTime<chrono::Utc> = date_time.try_into()?;

        let date_time = time::OffsetDateTime::from_unix_timestamp(date_time.timestamp())?;
        Ok(date_time)
    }
}
