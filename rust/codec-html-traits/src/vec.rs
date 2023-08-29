use common::itertools::Itertools;

use crate::prelude::*;

impl<T> ToHtml for Vec<T>
where
    T: ToHtml,
{
    fn to_html(&self) -> String {
        self.iter().map(|value| value.to_html()).join("")
    }
}
