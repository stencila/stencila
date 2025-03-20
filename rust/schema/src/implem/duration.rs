use common::{
    chrono, chrono_humanize,
    eyre::{OptionExt, Report},
    inflector::Inflector,
};

use crate::{prelude::*, Duration, TimeUnit};

impl Duration {
    /// Get a humanized representation of the duration
    pub fn humanize(&self, precise: bool) -> String {
        use chrono_humanize::{Accuracy, HumanTime, Tense};

        let duration: Result<chrono::Duration> = self.try_into();
        let Ok(duration) = duration else {
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

impl TryFrom<&Duration> for chrono::Duration {
    type Error = Report;

    fn try_from(duration: &Duration) -> Result<Self, Self::Error> {
        use chrono::Duration as CD;

        match &duration.time_unit {
            TimeUnit::Year => CD::try_days((duration.value as f64 * 365.25) as i64),
            TimeUnit::Month => CD::try_days((duration.value as f64 * 30.5) as i64),
            TimeUnit::Week => CD::try_weeks(duration.value),
            TimeUnit::Day => CD::try_days(duration.value * 86400),
            TimeUnit::Hour => CD::try_hours(duration.value * 3600),
            TimeUnit::Minute => CD::try_minutes(duration.value * 60),
            TimeUnit::Second => CD::try_seconds(duration.value),
            TimeUnit::Millisecond => CD::try_milliseconds(duration.value),
            TimeUnit::Microsecond => Some(CD::microseconds(duration.value)),
            TimeUnit::Nanosecond => Some(CD::nanoseconds(duration.value)),
            TimeUnit::Picosecond => Some(CD::nanoseconds(duration.value * 1_000)),
            TimeUnit::Femtosecond => Some(CD::nanoseconds(duration.value * 1_000_000)),
            TimeUnit::Attosecond => Some(CD::nanoseconds(duration.value * 1_000_000_000)),
        }
        .ok_or_eyre("Unable to convert Duration to chrono::Duration")
    }
}

impl From<&Duration> for time::Duration {
    fn from(duration: &Duration) -> Self {
        use time::Duration as TD;

        match &duration.time_unit {
            TimeUnit::Year => TD::days((duration.value as f64 * 365.25) as i64),
            TimeUnit::Month => TD::days((duration.value as f64 * 30.5) as i64),
            TimeUnit::Week => TD::weeks(duration.value),
            TimeUnit::Day => TD::days(duration.value),
            TimeUnit::Hour => TD::hours(duration.value),
            TimeUnit::Minute => TD::minutes(duration.value),
            TimeUnit::Second => TD::seconds(duration.value),
            TimeUnit::Millisecond => TD::milliseconds(duration.value),
            TimeUnit::Microsecond => TD::microseconds(duration.value),
            TimeUnit::Nanosecond => TD::nanoseconds(duration.value),
            TimeUnit::Picosecond => TD::nanoseconds(duration.value * 1_000),
            TimeUnit::Femtosecond => TD::nanoseconds(duration.value * 1_000_000),
            TimeUnit::Attosecond => TD::nanoseconds(duration.value * 1_000_000_000),
        }
    }
}
