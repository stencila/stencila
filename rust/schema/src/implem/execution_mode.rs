use crate::{prelude::*, ExecutionMode};

impl ExecutionMode {
    pub fn to_html_special(&self, _context: &mut HtmlEncodeContext) -> String {
        self.to_string().to_lowercase()
    }
}
