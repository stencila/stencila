use common::itertools::Itertools;

use crate::HtmlCodec;

impl<T> HtmlCodec for Vec<T>
where
    T: HtmlCodec,
{
    fn to_html(&self) -> String {
        self.iter().map(|value| value.to_html()).join("")
    }

    fn to_html_parts(&self) -> (&str, Vec<String>, Vec<String>) {
        // This implementation should never be called (because
        // `to_html` is implemented and `to_html_parts` is only
        // ever called on `options`).
        unreachable!("this method should not be called directly for vec")
    }

    fn to_html_attr(&self) -> String {
        // This implementation should rarely be called, but if it is, it
        // returns all the nodes as their attribute value representation
        // concatenated into a JSON array.
        [
            "[",
            &self.iter().map(|value| value.to_html_attr()).join(","),
            "]",
        ]
        .concat()
    }
}
