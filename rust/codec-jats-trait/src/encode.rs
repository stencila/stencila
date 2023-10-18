use std::fmt::Display;

use common::itertools::Itertools;

/// Escape text
pub fn escape<S>(unescaped: S) -> String
where
    S: AsRef<str>,
{
    let unescaped = unescaped.as_ref();
    let mut escaped = String::with_capacity(unescaped.len());

    for char in unescaped.chars() {
        match char {
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '\'' => escaped.push_str("&apos;"),
            '"' => escaped.push_str("&quot;"),
            '&' => escaped.push_str("&amp;"),

            '\t' => escaped.push_str("&#9;"),
            '\n' => escaped.push_str("&#10;"),
            '\r' => escaped.push_str("&#13;"),

            _ => escaped.push(char),
        }
    }

    escaped
}

/// Encode an element
pub fn elem<N, A, AN, AV, C>(name: N, attrs: A, content: C) -> String
where
    N: AsRef<str>,
    A: IntoIterator<Item = (AN, AV)>,
    AN: Display,
    AV: Display,
    C: AsRef<str>,
{
    let name = name.as_ref();

    if name.is_empty() {
        return content.as_ref().to_string();
    }

    let attrs = attrs
        .into_iter()
        .map(|(name, value)| format!("{name}=\"{value}\"", value = escape(value.to_string())))
        .join(" ");

    let content = content.as_ref();

    [
        "<",
        name,
        if attrs.is_empty() { "" } else { " " },
        &attrs,
        ">",
        content,
        "</",
        name,
        ">",
    ]
    .concat()
}

/// Encode an element with no attributes
pub fn elem_no_attrs<N, C>(name: N, content: C) -> String
where
    N: AsRef<str>,
    C: AsRef<str>,
{
    elem::<_, [(&str, &str); 0], _, _, _>(name, [], content)
}
