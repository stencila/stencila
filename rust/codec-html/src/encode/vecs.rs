//! Encode `Vec<>`s to HTML

use super::{EncodeContext, ToHtml};

impl<Type> ToHtml for Vec<Type>
where
    Type: ToHtml,
{
    fn to_html(&self, context: &EncodeContext) -> String {
        self.iter()
            .map(|item| item.to_html(context))
            .collect::<Vec<String>>()
            .concat()
    }
}
