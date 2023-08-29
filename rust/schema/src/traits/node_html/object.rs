use common::itertools::Itertools;

use codec_html_traits::to_html::{attr, elem, name, ToHtml};

use crate::Object;

impl ToHtml for Object {
    fn to_html(&self) -> String {
        elem(
            &name("Object"),
            &[],
            &[elem(
                "ul",
                &[],
                &self
                    .iter()
                    .map(|(key, value)| elem("li", &[attr("key", key)], &[value.to_html()]))
                    .collect_vec(),
            )],
        )
    }
}
