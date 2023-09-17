use common::itertools::Itertools;

/// Encode an element
pub fn elem<N, A, C>(name: N, attrs: A, content: C) -> String
where
    N: AsRef<str>,
    A: IntoIterator<Item = (String, String)>,
    C: AsRef<str>,
{
    let name = name.as_ref();
    
    if name.is_empty() {
        return content.as_ref().to_string()
    }
    
    let attrs = attrs
        .into_iter()
        .map(|(name, value)| format!("{name}=\"{}\"", value.replace('"', "&quot;")))
        .join(" ");

    [
        "<",
        name,
        if attrs.is_empty() { "" } else { " " },
        &attrs,
        ">",
        content.as_ref(),
        "</",
        name,
        ">",
    ]
    .concat()
}
