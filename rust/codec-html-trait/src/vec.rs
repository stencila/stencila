use common::itertools::Itertools;

use crate::prelude::*;

impl<T> HtmlCodec for Vec<T>
where
    T: HtmlCodec,
{
    fn to_html(&self) -> String {
        self.iter().map(|value| value.to_html()).join("")
    }
}
