use common::{chrono, chrono_humanize, inflector::Inflector};

use crate::{prelude::*, Duration, TimeUnit};

impl Duration {
    /// Get a humanized representation of the duration
    pub fn humanize(&self, precise: bool) -> String {
        use chrono::Duration;
        use chrono_humanize::{Accuracy, HumanTime, Tense};

        let duration = match self.time_unit {
            TimeUnit::Year => Duration::try_days((self.value as f64 * 365.25) as i64),
            TimeUnit::Month => Duration::try_days((self.value as f64 * 30.5) as i64),
            TimeUnit::Week => Duration::try_weeks(self.value),
            TimeUnit::Day => Duration::try_days(self.value * 86400),
            TimeUnit::Hour => Duration::try_hours(self.value * 3600),
            TimeUnit::Minute => Duration::try_minutes(self.value * 60),
            TimeUnit::Second => Duration::try_seconds(self.value),
            TimeUnit::Millisecond => Duration::try_milliseconds(self.value),
            TimeUnit::Microsecond => Some(Duration::seconds(self.value)),
            TimeUnit::Nanosecond => Some(Duration::nanoseconds(self.value)),
            TimeUnit::Picosecond => Some(Duration::nanoseconds(self.value * 1_000)),
            TimeUnit::Femtosecond => Some(Duration::nanoseconds(self.value * 1_000_000)),
            TimeUnit::Attosecond => Some(Duration::nanoseconds(self.value * 1_000_000_000)),
        };

        let Some(duration) = duration else {
            return "-".to_string();
        };

        let human_time = HumanTime::from(duration);
        human_time.to_text_en(
            if precise {
                Accuracy::Precise
            } else {
                Accuracy::Rough
            },
            Tense::Present,
        )
    }

    /// Encode a duration as a DOM HTML attribute
    ///
    /// This is lossy with respect to the `timeUnit` of the duration but produces
    /// a far more compact representation compared to the default JSON string
    pub fn to_dom_attr(name: &str, duration: &Self, context: &mut DomEncodeContext) {
        context.push_attr(&name.to_kebab_case(), &duration.value.to_string());
    }

    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        (
            elem(
                "duration",
                [
                    ("value", self.value.to_string()),
                    ("unit", self.time_unit.to_string()),
                ],
                format!("{} {}s", self.value, self.time_unit),
            ),
            Losses::none(),
        )
    }
}
