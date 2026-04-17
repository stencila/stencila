use crate::{DateTime, prelude::*};

/// Create curly-braced Markdown attrs for suggestion metadata
pub(crate) fn suggestion_attrs(
    authors: &Option<Vec<Author>>,
    date_published: &Option<DateTime>,
) -> Option<String> {
    let mut attrs = Vec::new();

    fn escape_attr_value(value: &str) -> String {
        value.replace('\\', "\\\\").replace('"', "\\\"")
    }

    if let Some(authors) = authors
        && !authors.is_empty()
    {
        let by = authors
            .iter()
            .map(|author| author.short_name())
            .collect::<Vec<_>>()
            .join("; ");

        attrs.push(format!(r#"by="{}""#, escape_attr_value(&by)));
    }

    if let Some(at) = date_published {
        attrs.push(format!(r#"at="{}""#, escape_attr_value(&at.value)));
    }

    (!attrs.is_empty()).then(|| format!(" {{{}}}", attrs.join(", ")))
}
