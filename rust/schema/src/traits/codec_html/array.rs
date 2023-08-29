use crate::Array;

use super::prelude::*;

impl HtmlCodec for Array {
    fn to_html(&self) -> String {
        elem(
            &name("Array"),
            &[],
            &[elem(
                "ol",
                &[],
                &self
                    .iter()
                    .map(|value| elem("li", &[], &[value.to_html()]))
                    .collect_vec(),
            )],
        )
    }
}
