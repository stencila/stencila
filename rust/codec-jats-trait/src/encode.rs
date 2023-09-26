use std::fmt::Display;

pub use quick_xml::escape::escape;

use common::itertools::Itertools;

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
        .map(|(name, value)| format!("{name}=\"{}\"", escape(&value.to_string())))
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
