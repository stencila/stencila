use codec_dom_trait::DomEncodeContext;
use common::inflector::Inflector;

use crate::CodeLocation;

impl CodeLocation {
    /// Encode a code location as a DOM HTML attribute
    ///
    /// Represents the location as a JSON array of integers as a more
    /// compact, and convenient, alternative to encoding a separate element.
    pub fn to_dom_attr(name: &str, location: &Self, context: &mut DomEncodeContext) {
        let mut array = "[".to_string();
        let mut append = |value: Option<u64>, last: bool| {
            array.push_str(&value.map_or_else(|| "-1".to_string(), |value| value.to_string()));
            array.push(if last { ']' } else { ',' })
        };
        append(location.start_line, false);
        append(location.start_column, false);
        append(location.end_line, false);
        append(location.end_column, true);

        context.push_attr(&name.to_kebab_case(), &array);
    }
}
