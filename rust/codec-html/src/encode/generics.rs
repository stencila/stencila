use codec::common::{serde::Serialize, serde_json};

use super::{attr, nothing, EncodeContext, ToHtml};

impl<T> ToHtml for Option<T>
where
    T: ToHtml,
{
    /// Encode an option as HTML
    ///
    /// Simply delegates to the optional type, returning an empty string
    /// if `None`
    fn to_html(&self, context: &mut EncodeContext) -> String {
        match self {
            Some(value) => value.to_html(context),
            None => nothing(),
        }
    }

    /// Encode an option as an HTML element attribute
    ///
    /// Simply delegates to the optional type, returning an empty string
    /// if `None`
    fn to_attr(&self, name: &str) -> String {
        match self {
            Some(value) => value.to_attr(name),
            None => nothing(),
        }
    }
}

impl<T> ToHtml for Box<T>
where
    T: ToHtml,
{
    /// Encode a box as HTML
    ///
    /// Simply delegates to the boxed type.
    fn to_html(&self, context: &mut EncodeContext) -> String {
        self.as_ref().to_html(context)
    }

    /// Encode a box as an HTML element attribute
    ///
    /// Simply delegates to the boxed type.
    fn to_attr(&self, name: &str) -> String {
        self.as_ref().to_attr(name)
    }
}

impl<T> ToHtml for Vec<T>
where
    T: ToHtml + Serialize,
{
    /// Encode a vector as HTML
    ///
    /// Encodes each item to HTML and concatenates
    fn to_html(&self, context: &mut EncodeContext) -> String {
        self.iter()
            .map(|item| item.to_html(context))
            .collect::<Vec<String>>()
            .concat()
    }

    /// Encode a vector as an HTML element attribute
    fn to_attr(&self, name: &str) -> String {
        attr(
            name,
            &serde_json::to_string(self).unwrap_or_else(|error| error.to_string()),
        )
    }
}
