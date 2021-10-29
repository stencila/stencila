use eyre::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use stencila_schema::{Node, Person};

/// Decode a string into a `Node::Person` variant.
///
/// Always returns an `Ok` result with a `Node::Person` value.
pub fn decode(input: &str) -> Result<Node> {
    Ok(Node::Person(decode_person(input)))
}

/// Decode a string to a `Person` struct.
///
/// Properties such as `given_names` and `family_names` are populated
/// if possible, but if parsing fails, falling back to `name` having
/// the provided input string.
pub fn decode_person(input: &str) -> Person {
    if let Some(person) = decode_person_maybe(input) {
        person
    } else {
        Person {
            name: Some(Box::new(input.to_string())),
            ..Default::default()
        }
    }
}

/// Attempt to decode a string to a `Person` struct.
///
/// Returns `Some(Person)` if parsing was successful, `None`  otherwise.
pub fn decode_person_maybe(input: &str) -> Option<Person> {
    if let Some(name) = human_name::Name::parse(input) {
        let given_names = if let Some(first_name) = name.given_name() {
            let mut given_names = vec![first_name.to_string()];
            if let Some(middle_names) = name.middle_names() {
                let mut middle_names = middle_names
                    .iter()
                    .map(|str| str.to_string())
                    .collect::<Vec<String>>();
                given_names.append(&mut middle_names)
            }
            Some(given_names)
        } else {
            None
        };

        let family_names = if name.surnames().is_empty() {
            None
        } else {
            Some(
                name.surnames()
                    .iter()
                    .map(|str| str.to_string())
                    .collect::<Vec<String>>(),
            )
        };

        // Note: currently not using the "generational suffix" e.g "Jr" parsed
        // by `human_name`.

        // Note: there is no easy way to extract honorific prefix and suffix
        // See https://github.com/djudd/human-name/issues/6

        static EMAIL_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new("<([^@]+@.+)>").expect("Unable to create regex"));
        let emails = EMAIL_REGEX
            .captures(input)
            .map(|email| vec![email[1].to_string()]);

        static URL_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new("\\((https?://[^)]+)\\)").expect("Unable to create regex"));
        let url = URL_REGEX
            .captures(input)
            .map(|url| Box::new(url[1].to_string()));

        Some(Person {
            emails,
            family_names,
            given_names,
            url,
            ..Default::default()
        })
    } else {
        None
    }
}
