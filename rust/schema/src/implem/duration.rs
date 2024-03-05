use common::inflector::Inflector;

use crate::{prelude::*, Duration};

impl Duration {
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
