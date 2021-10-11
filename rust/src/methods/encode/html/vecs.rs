use super::{Context, ToHtml};

impl<Type> ToHtml for Vec<Type>
where
    Type: ToHtml,
{
    fn to_html(&self, _slot: &str, context: &Context) -> String {
        self.iter()
            .map(|item| item.to_html("", context))
            .collect::<Vec<String>>()
            .concat()
    }
}
