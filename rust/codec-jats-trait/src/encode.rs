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
