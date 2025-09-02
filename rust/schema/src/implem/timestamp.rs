use eyre::{OptionExt, Report};
use inflector::Inflector;

use crate::{Duration, TimeUnit, Timestamp, prelude::*};

impl Timestamp {
    /// Get the current timestamp
    pub fn now() -> Self {
        Self {
            value: chrono::Utc::now().timestamp_millis(),
            time_unit: TimeUnit::Millisecond,
            ..Default::default()
        }
    }

    /// Get the duration since another timestamp
    pub fn duration(&self, other: &Self) -> Result<Duration> {
        if self.time_unit != other.time_unit {
            bail!(
                "Time units of timestamps are different {} != {}",
                self.time_unit,
                other.time_unit
            )
        }

        if self.value < other.value {
            bail!("Other timestamp should be before this timestamp")
        }

        Ok(Duration {
            value: self.value - other.value,
            time_unit: self.time_unit,
            ..Default::default()
        })
    }

    /// Get a humanized representation of the timestamp
    pub fn humanize(&self, precise: bool) -> String {
        use chrono::{DateTime, Utc};
        use chrono_humanize::{Accuracy, HumanTime, Tense};

        let date_time: Result<DateTime<Utc>> = self.try_into();
        let Ok(date_time) = date_time else {
            return "-".to_string();
        };

        let human_time = HumanTime::from(date_time);
        human_time.to_text_en(
            if precise {
                Accuracy::Precise
            } else {
                Accuracy::Rough
            },
            Tense::Past,
        )
    }

    /// Encode a timestamp as a DOM HTML attribute
    ///
    /// This is lossy with respect to the `timeUnit` of the timestamp but produces
    /// a far more compact representation compared to the default JSON string
    pub fn to_dom_attr(name: &str, timestamp: &Self, context: &mut DomEncodeContext) {
        context.push_attr(&name.to_kebab_case(), &timestamp.value.to_string());
    }

    pub fn to_jats_special(&self) -> (String, Losses) {
        use stencila_codec_jats_trait::encode::elem;

        (
            elem(
                "timestamp",
                [("value", &self.value)],
                self.value.to_string(),
            ),
            Losses::none(),
        )
    }
}

impl TryFrom<&Timestamp> for chrono::DateTime<chrono::Utc> {
    type Error = Report;

    fn try_from(ts: &Timestamp) -> Result<Self, Self::Error> {
        use chrono::DateTime;

        match ts.time_unit {
            TimeUnit::Year => DateTime::from_timestamp(ts.value * 31557600, 0),
            TimeUnit::Month => DateTime::from_timestamp(ts.value * 2635200, 0),
            TimeUnit::Week => DateTime::from_timestamp(ts.value * 604800, 0),
            TimeUnit::Day => DateTime::from_timestamp(ts.value * 86400, 0),
            TimeUnit::Hour => DateTime::from_timestamp(ts.value * 3600, 0),
            TimeUnit::Minute => DateTime::from_timestamp(ts.value * 60, 0),
            TimeUnit::Second => DateTime::from_timestamp(ts.value, 0),
            TimeUnit::Millisecond => DateTime::from_timestamp_millis(ts.value),
            TimeUnit::Microsecond => DateTime::from_timestamp_micros(ts.value),
            TimeUnit::Nanosecond => Some(DateTime::from_timestamp_nanos(ts.value)),
            TimeUnit::Picosecond => Some(DateTime::from_timestamp_nanos(ts.value / 1_000)),
            TimeUnit::Femtosecond => Some(DateTime::from_timestamp_nanos(ts.value / 1_000_000)),
            TimeUnit::Attosecond => Some(DateTime::from_timestamp_nanos(ts.value / 1_000_000_000)),
        }
        .ok_or_eyre("Unable to convert timestamp to chrono::DateTime")
    }
}

impl TryFrom<&Timestamp> for time::OffsetDateTime {
    type Error = Report;

    fn try_from(ts: &Timestamp) -> Result<Self, Self::Error> {
        use time::OffsetDateTime;

        Ok(match ts.time_unit {
            TimeUnit::Year => OffsetDateTime::from_unix_timestamp(ts.value * 31557600),
            TimeUnit::Month => OffsetDateTime::from_unix_timestamp(ts.value * 2635200),
            TimeUnit::Week => OffsetDateTime::from_unix_timestamp(ts.value * 604800),
            TimeUnit::Day => OffsetDateTime::from_unix_timestamp(ts.value * 86400),
            TimeUnit::Hour => OffsetDateTime::from_unix_timestamp(ts.value * 3600),
            TimeUnit::Minute => OffsetDateTime::from_unix_timestamp(ts.value * 60),
            TimeUnit::Second => OffsetDateTime::from_unix_timestamp(ts.value),
            TimeUnit::Millisecond => {
                OffsetDateTime::from_unix_timestamp_nanos(ts.value as i128 * 1_000_000)
            }
            TimeUnit::Microsecond => {
                OffsetDateTime::from_unix_timestamp_nanos(ts.value as i128 * 1_000)
            }
            TimeUnit::Nanosecond => OffsetDateTime::from_unix_timestamp_nanos(ts.value as i128),
            TimeUnit::Picosecond => {
                OffsetDateTime::from_unix_timestamp_nanos(ts.value as i128 / 1_000)
            }
            TimeUnit::Femtosecond => {
                OffsetDateTime::from_unix_timestamp_nanos(ts.value as i128 / 1_000_000)
            }
            TimeUnit::Attosecond => {
                OffsetDateTime::from_unix_timestamp_nanos(ts.value as i128 / 1_000_000_000)
            }
        }?)
    }
}
