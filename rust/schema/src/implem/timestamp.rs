use common::inflector::Inflector;

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
