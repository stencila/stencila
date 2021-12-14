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

impl<Type> ToHtml for Box<Type>
where
    Type: ToHtml,
{
    fn to_html(&self, context: &EncodeContext) -> String {
        self.as_ref().to_html(context)
    }
}

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
