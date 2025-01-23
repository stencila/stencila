use codec::schema::{Person, Primitive, PropertyValue, PropertyValueOrString};
use common::{regex, tracing};
use std::{borrow::Cow, sync::OnceLock};

pub(crate) fn extract_affiliations(author: &Person) -> Option<impl Iterator<Item = Cow<str>>> {
    let Person {
        affiliations: Some(aff),
        ..
    } = author
    else {
        return None;
    };

    if aff.is_empty() {
        return None;
    }

    if aff.len() > 1 {
        common::tracing::warn!("Author has multiple affiliations, only one can be added programmatically. Edit the record in Zenodo's web interface to correct any mistakes.");
    }

    let aff = aff
        .iter()
        .filter_map(|org| org.name.as_ref())
        .filter(|name| name.is_empty())
        .map(Cow::from);

    Some(aff)
}

/// Extract the name of a person from a `Person` object in a way that complies
/// with Zenodo's standard.
///
/// Zenodo requires that names are provided as `<family-name>, <given-name>`.
/// This conflicts with Stencila's more flexible schema, which allows for
/// multiple family and given names. To reconcile this, when multiple given
/// names are available, they are concatenated with a hyphen. When multiple
/// first names are available, they are concatenated with a space.
///
/// Lastly, we might have parts that are missing. To address the, we return
/// `<family-name>` or `<given-name>`, if that's all that's available.
pub(crate) fn extract_name(person: &Person) -> Option<Cow<str>> {
    // join multiple family names with a hyphen
    let family_names = person.family_names.as_ref().and_then(|names| {
        names.iter().map(Cow::from).reduce(|mut parts, s| {
            let p = parts.to_mut();
            p.push('-');
            p.push_str(&s);
            parts
        })
    });

    // join multiple given names with a space
    let given_names = person.given_names.as_ref().and_then(|names| {
        names.iter().map(Cow::from).reduce(|mut parts, s| {
            let p = parts.to_mut();
            p.push(' ');
            p.push_str(&s);
            parts
        })
    });

    match (family_names, given_names) {
        (Some(family), Some(given)) => {
            Some(format!("{}, {}", family, given).into()) // Format prescribed by Zenodo
        }
        (None, Some(given)) => Some(given),
        (Some(family), None) => Some(family),
        _ => {
            tracing::warn!("Author has no name");
            None
        }
    }
}

pub(crate) fn extract_doi(id: &PropertyValueOrString) -> Option<Cow<str>> {
    match id {
        PropertyValueOrString::PropertyValue(property) => {
            let PropertyValue {
                property_id: Some(property_id),
                value: Primitive::String(value),
                ..
            } = property
            else {
                return None;
            };

            if property_id.as_str() == "https://registry.identifiers.org/registry/doi" {
                return Some(Cow::Borrowed(value));
            };

            find_doi(value)
        }
        PropertyValueOrString::String(text) => find_doi(text),
    }
}

/// Returns the first DOI pattern, e.g. "10.1126/science.1115581", within input text.
fn find_doi(text: &str) -> Option<Cow<str>> {
    static DOI_REGEX: OnceLock<regex::Regex> = OnceLock::new();

    #[allow(clippy::unwrap_used)]
    let searcher = DOI_REGEX.get_or_init(|| regex::Regex::new(r"\b(10\.\d{4,5}/\S+)\b").unwrap());

    searcher.find(text).map(|m| Cow::Borrowed(m.as_str()))
}

#[common::tracing::instrument]
pub(crate) fn extract_orcid(id: &PropertyValueOrString) -> Option<Cow<str>> {
    match id {
        PropertyValueOrString::PropertyValue(property) => {
            let PropertyValue {
                property_id: Some(property_id),
                value: Primitive::String(value),
                ..
            } = property
            else {
                return None;
            };

            if property_id.as_str().contains("orcid") {
                return find_orcid(value);
            };

            find_orcid(value.as_str())
        }
        PropertyValueOrString::String(text) => find_orcid(text),
    }
}

#[common::tracing::instrument]
pub(crate) fn find_orcid(text: &str) -> Option<Cow<str>> {
    // this function is uglier than it should be because we permit people to specify ORCID
    // in a variety of ways, including with or without the orcid.org domain, with or without
    // dashes, and with or without the final checksum digit.

    static ORCID_REGEX: OnceLock<regex::Regex> = OnceLock::new();

    #[allow(clippy::unwrap_used)]
    let searcher = ORCID_REGEX
        .get_or_init(|| regex::Regex::new(r"(\d{4})-?(\d{4})-?(\d{4})-?(\d{3}[0-9X])").unwrap());

    let cap = searcher.captures(text)?;
    let (full, [a, b, c, d]) = cap.extract();

    // if hyphens are all in the right place, then exit early
    let bytes = full.as_bytes();
    if bytes[4] == b'-' && bytes[9] == b'-' && bytes[14] == b'-' {
        return Some(Cow::Borrowed(full));
    }

    // ... otherwise insert them
    Some(Cow::Owned(format!("{}-{}-{}-{}", a, b, c, d)))
}

#[cfg(test)]
mod test_helpers {
    use super::find_orcid;

    #[test]
    fn find_orcid_detects_correctly_formatted_orcids() {
        let orcids = ["0000-0002-1825-1111", "0000-0002-1825-009X"];

        for orcid in orcids {
            let expected = Some(orcid);
            let actual = find_orcid(orcid);
            assert_eq!(
                actual.as_deref(),
                expected,
                "input: {orcid:?} failed to produce {expected:?}"
            );
        }
    }

    #[test]
    fn find_orcid_detects_malformed_orcids() {
        let orcid = "0000000218253333";
        let actual = find_orcid(orcid);
        let expected = Some("0000-0002-1825-3333");
        assert_eq!(
            actual.as_deref(),
            expected,
            "input: {orcid:?} failed to produce {expected:?}"
        )
    }

    #[test]
    fn find_orcid_detects_orcids_in_urls() {
        let urls = [
            (
                "http://orcid.org/0000-0002-2494-2700",
                Some("0000-0002-2494-2700"),
            ),
            (
                "https://orcid.org/0000-0002-1825-2222",
                Some("0000-0002-1825-2222"),
            ),
            (
                "https://orcid.org/0000-0002-1825-009X/other",
                Some("0000-0002-1825-009X"),
            ),
            (
                "https://orcid.org/0000-0002-1825-4444/",
                Some("0000-0002-1825-4444"),
            ),
            ("https://orcid.org", None),
        ];

        for (url, expected) in urls {
            let actual = find_orcid(url);
            assert_eq!(
                actual.as_deref(),
                expected,
                "input: {url:?} failed to produce {expected:?}"
            );
        }
    }
}
