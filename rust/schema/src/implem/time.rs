use crate::{Time, prelude::*};

impl JatsCodec for Time {
    fn to_jats(&self, context: &mut JatsEncodeContext) {
        context
            .enter_elem("time")
            .push_attr("iso-8601-time", &self.value)
            .push_text(&self.value)
            .exit_elem();
    }
}
