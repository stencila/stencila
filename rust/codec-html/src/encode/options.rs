//! Encode `Option<>`s to HTML

use super::{EncodeContext, ToHtml};

impl<Type> ToHtml for Option<Type>
where
    Type: ToHtml,
{
    fn to_html(&self, context: &EncodeContext) -> String {
        match self {
            Some(value) => value.to_html(context),
            None => "".to_string(),
        }
    }
}
