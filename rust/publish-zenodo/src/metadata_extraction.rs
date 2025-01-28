use codec::schema::{Person, Primitive, PropertyValue, PropertyValueOrString};
use common::{eyre::OptionExt, regex, tracing};
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

/// Parse an input from the command line as a Ghost host
pub fn parse_doi(arg: &str) -> common::eyre::Result<String> {
    let doi = find_doi(arg).ok_or_eyre("DOI supplied is invalid")?;
    tracing::error!("wat {}", doi);
    Ok(doi.to_string())
}


/// Returns the first DOI pattern, e.g. "10.1126/science.1115581", within input text.
fn find_doi(text: &str) -> Option<Cow<str>> {
    // Matches only ASCII values for performance
    static DOI_ASCII_REGEX: OnceLock<regex::Regex> = OnceLock::new();
    static DOI_UNICODE_REGEX: OnceLock<regex::Regex> = OnceLock::new();

    // From the [DOI Handbook]
    //
    // > General Characteristics of the [DOI Syntax]
    // >
    // > The DOI syntax shall be made up of a DOI prefix and a DOI suffix
    // > separated by a forward slash.
    // >
    // > There is no defined limit on the length of the DOI name, or of the DOI
    // > prefix or DOI suffix.
    // >
    // > The DOI name is case-insensitive and can incorporate any printable
    // > characters from the legal graphic characters of Unicode. Further
    // > constraints on character use (for example, use of language-specific
    // > alphanumeric characters) can be defined for an application by the ISO
    // > 26324 Registration Authority.
    // >
    // > NOTE:  The exact definition of case-insensitivity is expected to be
    // > clarified in an upcoming revision of the International Standard.
    //
    // >  DOI Prefix
    // >
    // >  The DOI prefix is <directoryIndicator>.<registrantCode>.
    // >
    // >  The following rules apply:
    // >  • The directory indicator can contain only numeric values. The directory
    // >    indicator is usually 10 but other indicators may be designated as
    // >    compliant by the DOI Foundation.
    // >  • The registrant code can contain only numeric values and one or
    // >    several full stops which are used to subdivide the code. For example:
    // >    10.1000, 10.500.100, etc.
    // >  • If the directory indicator is 10 then a registrant code is mandatory.
    //
    // > DOI Suffix
    // >
    // > [...]
    // >
    // > No length limit is set to the suffix by the DOI System.
    // >
    // > Example 1 10.1000/123456: DOI name with the DOI prefix "10.1000" and the
    // > DOI suffix "123456"
    // >
    // > Example 2 10.1038/issn.1476-4687: DOI suffix using an ISSN To construct a
    // > DOI suffix using an ISSN, precede the ISSN (including the hyphen) with
    // > the lowercase letters "issn" and a period, as in this hypothetical
    // > example of a DOI name for the electronic version of the scientific
    // > journal Nature.
    //
    // [DOI Handbook]: https://www.doi.org/doi-handbook/HTML/
    //
    // From https://www.iana.org/assignments/urn-formal/doi
    //
    // > The 2022 edition of ISO 26324 has amended the syntax of the prefix by
    // > removing the requirement for the directory indicator to be "10" and
    // > allow also DOI names without a registrant code.

    // Fast path - most docs are not going to need all of unicode,
    // so we can just check the ASCII range.
    //
    // CrossRef (2015) reports that this catches 99.3% of DOIs:
    // https://www.crossref.org/blog/dois-and-matching-regular-expressions/
    #[allow(clippy::unwrap_used)]
    let ascii = DOI_ASCII_REGEX.get_or_init(|| {
        regex::Regex::new(
            r"(?x)
            \b  # start at a word boundary, rather than the middle of a word

            (  # start of capture group

            # if the directory indicator is '10', then it must be followed
            # by a dot, then a numeric registrant code.
            #
            # either match 10.N or N followed by numbers and full stops
            # (if 10, then there must be a trailing dot)
            (?:10\.[0-9]+(?:\.[0-9]+)*|[0-9]+(?:\.[0-9]+)*)

            /  # literal /

            [[[:alnum:]][[:punct:]]]+

            ) # end of capture group

            (?-u:\b|\s|$)
        ",
        )
        .unwrap()
    });

    if let Some(captures) = ascii.captures(text) {
        let (_full, [doi]) = captures.extract();
        return Some(Cow::Borrowed(doi));
    }

    #[allow(clippy::unwrap_used)]
    let all_unicode = DOI_UNICODE_REGEX.get_or_init(|| {
        regex::Regex::new(
            r"(?x)
            \b

            (

            (?:10\.[0-9]+(?:\.[0-9]+)*|[0-9]+(?:\.[0-9]+)*)

            /

            # all graphical characters, excluding whitespace
            (?:[[[:graph:]]&&[^\s]])+

            )

            (?:\b|\s|$)
        ",
        )
        .unwrap()
    });

    let captures = all_unicode.captures(text)?;
    let (_full, [doi]) = captures.extract();
    Some(Cow::Borrowed(doi))
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
mod dois {
    use super::*;

    #[test]
    fn can_extract_doi_from_structured_propval() {
        let property = PropertyValue {
            property_id: Some("https://registry.identifiers.org/registry/doi".to_string()),
            value: Primitive::String("10.1126/science.1115581".to_string()),
            ..Default::default()
        };
        let id = PropertyValueOrString::PropertyValue(property);

        assert_eq!(extract_doi(&id).as_deref(), Some("10.1126/science.1115581"));
    }

    #[test]
    fn can_extract_doi_from_string_only_propval() {
        let id = PropertyValueOrString::String("10.1234/example.123".to_string());

        assert_eq!(extract_doi(&id).as_deref(), Some("10.1234/example.123"));
    }

    #[test]
    fn valid_dois() {
        let dois = [
            "10.1126/science.1115581",
            "10.1234/example.doi",
            "10.12345/very.long.doi.string",
            "10.1234.567.89/sub-directory",
            "12/iso.26324-2022-compliant",
            "10.1016/S0735-1097(98)00347-7",
        ];

        for doi in dois {
            assert_eq!(find_doi(doi).as_deref(), Some(doi), "input = {doi}");
        }
    }

    #[test]
    fn doi_prefix_okay() {
        assert_eq!(
            find_doi("doi:10.1126/bbb.222").as_deref(),
            Some("10.1126/bbb.222")
        )
    }

    #[test]
    fn find_doi_in_urls() {
        assert_eq!(
            find_doi("https://doi.org/10.1126/science.1115581").as_deref(),
            Some("10.1126/science.1115581")
        );
        assert_eq!(
            find_doi("http://dx.doi.org/10.1234/example.123").as_deref(),
            Some("10.1234/example.123")
        );
        assert_eq!(
            find_doi("http://dx.doi.org/10.1038/issn.1476-4687").as_deref(),
            Some("10.1038/issn.1476-4687")
        );
        assert_eq!(
            find_doi("https://dx.doi.org/10.1038/issn.1476-4687").as_deref(),
            Some("10.1038/issn.1476-4687")
        );
    }

    #[test]
    fn test_extract_doi_invalid_property() {
        let property = PropertyValue {
            property_id: None,
            value: Primitive::String("10.1126/science.1115581".to_string()),
            ..Default::default()
        };
        let id = PropertyValueOrString::PropertyValue(property);

        assert_eq!(extract_doi(&id).as_deref(), None);
    }
}

#[cfg(test)]
mod doi_proptests {
    use super::*;

    use common_dev::proptest::prelude::*;

    // Strategy to generate valid DOI prefixes
    fn doi_prefix() -> impl Strategy<Value = String> {
        prop_oneof![
            // Classic "10.NNNN" format
            "[0-9]{4,10}".prop_map(|n| format!("10.{}", n)),
            // New ISO 26324 compliant format allowing other directory indicators
            "[0-9]{2,5}".prop_map(|n| n),
            // Nested
            r"[0-9]{2,5}\.[0-9]{4,10}(\.[0-9]{1,3})?(\.[0-9]{1,3})?".prop_map(|x| x),
        ]
    }

    // Strategy to generate common
    fn doi_suffix() -> impl Strategy<Value = String> {
        // graphical characters excluding whitespace
        r"[a-zA-Z-_/#()]{1,50}".prop_map(|x| x)
    }

    // Strategy to generate difficult, but technically valid, DOI suffixes
    fn unicode_doi_suffix() -> impl Strategy<Value = String> {
        // graphical characters excluding whitespace
        r"[[[:graph:]]&&[^\s]]{1,50}".prop_map(|x| x)
    }

    // Strategy to generate full valid DOIs
    fn valid_doi() -> impl Strategy<Value = String> {
        let suffix = prop_oneof![unicode_doi_suffix(), doi_suffix(),];

        (doi_prefix(), suffix).prop_map(|(prefix, suffix)| format!("{}/{}", prefix, suffix))
    }

    // Generate URL-like wrappers for DOIs
    fn part_of_url() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("https://doi.org/".to_string()),
            Just("http://dx.doi.org/".to_string()),
            Just("doi:".to_string()),
            Just("".to_string()),
        ]
    }

    proptest! {
        // Test that valid DOIs are properly extracted
        #[test]
        fn can_identify_valid_dois(doi in valid_doi()) {
            let result = find_doi(&doi);
            prop_assert_eq!(result.as_deref(), Some(doi.as_str()));
        }

        // Test that DOIs are properly extracted from URLs and other common formats
        #[test]
        fn find_doi_within_url(
            doi in valid_doi(),
            url_fragment in part_of_url()
        ) {
            let input = format!("{}{}", url_fragment, doi);
            let result = find_doi(&input);
            prop_assert_eq!(result.as_deref(), Some(doi.as_str()));
        }

        // Test that DOIs are found even with surrounding text
        #[test]
        fn find_doi_in_text(
            doi in valid_doi(),
            prefix in "[\\w]{0,50}\\s",
            suffix in "\\s[\\w]{0,50}"
        ) {
            let text = format!("{} {} {}", prefix, doi, suffix);
            let result = find_doi(&text);
            prop_assert_eq!(result.as_deref(), Some(doi.as_str()));
        }

        // Test that invalid text without DOIs returns None
        #[test]
        fn find_no_doi_in_invalid_text(
            s in "[^/0-9]{0,100}"
        ) {
            let result = find_doi(&s);
            prop_assert_eq!(result, None);
        }
    }
}

#[cfg(test)]
mod orcids {
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
