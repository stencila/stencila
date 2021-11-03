//! A codec for people's names, honorifics and contact details

use codec_trait::{
    eyre::{bail, Result},
    stencila_schema::{Node, Person},
    Codec, DecodeOptions,
};
use human_name::Name;
use once_cell::sync::Lazy;
use regex::Regex;

pub struct PersonCodec {}

impl Codec for PersonCodec {
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
            bail!("Unable to decode as a `Person`: {}", str)
        }
    }
}
