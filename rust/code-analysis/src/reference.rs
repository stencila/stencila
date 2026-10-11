/// Check whether a reference has a URI scheme that should not be localized.
pub fn has_non_local_uri_scheme(reference: &str) -> bool {
    let Some(colon) = reference.find(':') else {
        return false;
    };

    let scheme = &reference[..colon];
    if scheme.contains(['/', '?', '#']) {
        return false;
    }

    let mut chars = scheme.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !first.is_ascii_alphabetic()
        || !chars.all(|char| char.is_ascii_alphanumeric() || matches!(char, '+' | '-' | '.'))
    {
        return false;
    }

    let rest = &reference[(colon + 1)..];
    if rest.starts_with("//") || rest.starts_with('\\') {
        return true;
    }

    matches!(
        scheme.to_ascii_lowercase().as_str(),
        "data"
            | "doi"
            | "file"
            | "ftp"
            | "ftps"
            | "gs"
            | "http"
            | "https"
            | "mailto"
            | "s3"
            | "urn"
    )
}
