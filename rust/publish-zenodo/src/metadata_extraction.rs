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
        .map(|name| Cow::from(name));

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
    let searcher = ORCID_REGEX.get_or_init(|| {
        regex::Regex::new(r"(?:(?i:orcid\.org/)?)((\d{4})-?(\d{4})-?(\d{4})-(\d{3}[0-9X]))")
            .unwrap()
    });

    text.split('/')
        .last()
        .and_then(|id| {
            searcher.captures(id).map(|cap| {
                let (_incl_orcid_domain, [full, a, b, c, d]) = cap.extract();

                if full.contains('-') {
                    return Cow::Borrowed(full);
                }

                Cow::Owned(format!("{}-{}-{}-{}", a, b, c, d))
            })
        })
        .filter(|id| id != "0000-0002-1825-0097") // exclude test ORCID
}

#[cfg(test)]
mod test_helpers {
    use super::find_orcid;

    #[test]
    fn test_extract_orcid() {
        let patterns = [
            (
                "http://orcid.org/0000-0002-2494-2700",
                Some("0000-0002-2494-2700"),
            ),
            (
                "https://orcid.org/0000-0002-1825-009X",
                Some("0000-0002-1825-009X"),
            ),
            (
                "https://orcid.org/000000021825009X",
                Some("0000-0002-1825-009X"),
            ),
            ("0000-0002-1825-009X", Some("0000-0002-1825-009X")),
            ("https://orcid.org/0000-0002-1825-0097/", None),
            (
                "https://orcid.org/0000-0002-1825-009X/other",
                Some("0000-0002-1825-009X"),
            ),
            ("https://orcid.org", None),
        ];

        for (pattern, expected_outcome) in patterns {
            assert_eq!(find_orcid(pattern).as_deref(), expected_outcome);
        }
    }
}
