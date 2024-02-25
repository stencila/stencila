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
