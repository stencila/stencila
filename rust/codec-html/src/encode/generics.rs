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

    fn to_attrs(&self, context: &EncodeContext) -> Vec<String> {
        match self {
            Some(value) => value.to_attrs(context),
            None => Vec::new(),
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

    fn to_attrs(&self, context: &EncodeContext) -> Vec<String> {
        self.as_ref().to_attrs(context)
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

    fn to_attrs(&self, context: &EncodeContext) -> Vec<String> {
        self.iter()
            .flat_map(|item| item.to_attrs(context))
            .collect()
    }
}
