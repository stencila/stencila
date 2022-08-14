/// Escape characters in a string that is to be inserted in Markdown
///
/// This only escapes characters that may be present in inline Markdown
/// to avoid string in curly attributes etc from being parsed as
/// emphasis, strong etc. The list of escaped characters may
/// need to be augmented in the future.
///
/// Previously we also escaped '[' but that is often used in JSON
/// values for enum and array parameters so is now not escaped.
pub(crate) fn escape(string: &str) -> String {
    string
        .replace('_', "\\_")
        .replace('*', "\\*")
        .replace('$', "\\$")
}

/// Unescape characters
///
/// See [`escape`] for which characters need to be unescaped.
pub(crate) fn unescape(string: &str) -> String {
    string
        .replace("\\_", "_")
        .replace("\\*", "*")
        .replace("\\$", "$")
}
