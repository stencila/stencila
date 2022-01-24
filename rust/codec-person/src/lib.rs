use codec::{
    eyre::{bail, Result},
    stencila_schema::{Node, Person},
    utils::vec_string,
    Codec, CodecTrait, DecodeOptions, EncodeOptions,
};
use human_name::Name;
use once_cell::sync::Lazy;
use regex::Regex;

// A codec for people's names, honorifics and contact details
pub struct PersonCodec {}

impl CodecTrait for PersonCodec {
    fn spec() -> Codec {
        Codec {
            status: "beta".to_string(),
            formats: vec_string!["person"],
            root_types: vec_string!["Person"],
            ..Default::default()
        }
    }

    fn from_str(str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        if let Some(name) = Name::parse(str) {
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
                .captures(str)
                .map(|email| vec![email[1].to_string()]);

            static URL_REGEX: Lazy<Regex> =
                Lazy::new(|| Regex::new("\\((https?://[^)]+)\\)").expect("Unable to create regex"));
            let url = URL_REGEX
                .captures(str)
                .map(|url| Box::new(url[1].to_string()));

            Ok(Node::Person(Person {
                emails,
                family_names,
                given_names,
                url,
                ..Default::default()
            }))
        } else {
            bail!("Unable to decode string to a `Person`: {}", str)
        }
    }

    fn to_string(node: &Node, _options: Option<EncodeOptions>) -> Result<String> {
        let person = match node {
            Node::Person(person) => person,
            _ => bail!("Unable to encode node that is not a `Person`"),
        };

        let mut string = String::new();
        if let Some(honorific_prefix) = person.honorific_prefix.as_deref() {
            string.push_str(honorific_prefix);
        }
        if let Some(given_names) = &person.given_names {
            string.push(' ');
            string.push_str(&given_names.join(" "));
        }
        if let Some(family_names) = &person.family_names {
            string.push(' ');
            string.push_str(&family_names.join(" "));
        }
        if let Some(honorific_suffix) = person.honorific_suffix.as_deref() {
            string.push(' ');
            string.push_str(honorific_suffix);
        }
        if let Some(emails) = &person.emails {
            string.push_str(" <");
            string.push_str(&emails.join(" "));
            string.push('>');
        }
        if let Some(url) = person.url.as_deref() {
            string.push_str(" (");
            string.push_str(url);
            string.push(')');
        }

        Ok(string.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::assert_json_eq;

    /// Fairly trivial tests at present given that we're relying on `human_name` crate
    /// for parsing.
    #[test]
    fn from_str() -> Result<()> {
        assert_json_eq!(
            PersonCodec::from_str("Joe Bloggs", None)?,
            Node::Person(Person {
                given_names: Some(vec_string!["Joe"]),
                family_names: Some(vec_string!["Bloggs"]),
                ..Default::default()
            })
        );
        assert_json_eq!(
            PersonCodec::from_str(
                "Mary Mavis Maven <mary@example.com> (https://mary.example.com)",
                None
            )?,
            Node::Person(Person {
                given_names: Some(vec_string!["Mary", "Mavis"]),
                family_names: Some(vec_string!["Maven"]),
                emails: Some(vec_string!["mary@example.com"]),
                url: Some(Box::new("https://mary.example.com".to_string())),
                ..Default::default()
            })
        );
        Ok(())
    }

    #[test]
    fn to_string() -> Result<()> {
        assert_eq!(
            PersonCodec::to_string(
                &Node::Person(Person {
                    given_names: Some(vec_string!["Joe"]),
                    family_names: Some(vec_string!["Bloggs"]),
                    ..Default::default()
                }),
                None
            )?,
            "Joe Bloggs"
        );
        assert_eq!(
            PersonCodec::to_string(
                &Node::Person(Person {
                    given_names: Some(vec!["Mary".to_string(), "Mavis".to_string()]),
                    family_names: Some(vec_string!("Maven")),
                    emails: Some(vec_string!("mary@example.com")),
                    url: Some(Box::new("https://mary.example.com".to_string())),
                    ..Default::default()
                }),
                None
            )?,
            "Mary Mavis Maven <mary@example.com> (https://mary.example.com)"
        );
        Ok(())
    }
}
