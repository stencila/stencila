use common::{chrono, chrono_humanize, inflector::Inflector};

use crate::{prelude::*, Duration, TimeUnit, Timestamp};

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
            time_unit: self.time_unit.clone(),
            ..Default::default()
        })
    }

    /// Get a humanized representation of the timestamp
    pub fn humanize(&self, precise: bool) -> String {
        use chrono::DateTime;
        use chrono_humanize::{Accuracy, HumanTime, Tense};

        let date_time = match self.time_unit {
            TimeUnit::Year => DateTime::from_timestamp(self.value * 31557600, 0),
            TimeUnit::Month => DateTime::from_timestamp(self.value * 2635200, 0),
            TimeUnit::Week => DateTime::from_timestamp(self.value * 604800, 0),
            TimeUnit::Day => DateTime::from_timestamp(self.value * 86400, 0),
            TimeUnit::Hour => DateTime::from_timestamp(self.value * 3600, 0),
            TimeUnit::Minute => DateTime::from_timestamp(self.value * 60, 0),
            TimeUnit::Second => DateTime::from_timestamp(self.value, 0),
            TimeUnit::Millisecond => DateTime::from_timestamp_millis(self.value),
            TimeUnit::Microsecond => DateTime::from_timestamp_micros(self.value),
            TimeUnit::Nanosecond => Some(DateTime::from_timestamp_nanos(self.value)),
            TimeUnit::Picosecond => Some(DateTime::from_timestamp_nanos(self.value * 1_000)),
            TimeUnit::Femtosecond => Some(DateTime::from_timestamp_nanos(self.value * 1_000_000)),
            TimeUnit::Attosecond => {
                Some(DateTime::from_timestamp_nanos(self.value * 1_000_000_000))
            }
        };

        let Some(date_time) = date_time else {
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
        use codec_jats_trait::encode::elem;

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
