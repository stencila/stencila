pub use codec::CodecTrait;
use codec::{
    common::{
        eyre::{bail, eyre, Result},
        serde_json,
    },
    stencila_schema::{
        Article, CreativeWorkAuthors, CreativeWorkPublisher, CreativeWorkTypes, Date,
        InlineContent, Node, Organization, Periodical, Person, PublicationIssue,
        PublicationIssueIssueNumber, PublicationVolume, PublicationVolumeVolumeNumber,
        ThingIdentifiers,
    },
    utils::vec_string,
    Codec, DecodeOptions,
};

/// A codec for Citation Style Language (CSL) JSON
///
/// This uses `serde_json` to parse the JSON into a hash map and then extracts
/// various properties from it to populate an instance of one of the `CreativeWork` types,
/// for example, an `Article`, `Book` or `Dataset`.
///
/// The `citeproc-io` crate, which is a part of https://github.com/zotero/citeproc-rs, could be
/// used for parsing CSL-JSON:
///
///    let reference: citeproc_io::Reference = serde_json::from_str(str)?;
///
/// but it was found to be quite strict (e.g did not handle `"type": "journal-article"`)
/// and the parsed `Reference` still needed to be translated to a Stencila node.
pub struct CslCodec;

impl CslCodec {
    pub fn from_json(data: serde_json::Value) -> Result<Node> {
        let mut article = Article::default();

        if let Some(title) = data.get("title") {
            article.title = Some(vec![InlineContent::String(decode_string(title)?)]);
        }

        if let Some(date) = data.get("published") {
            article.date_published = Some(decode_date(date).map(Box::new)?);
        }
        if let Some(date) = data.get("submitted") {
            article.date_received = Some(decode_date(date).map(Box::new)?);
        }

        if let Some(authors) = data.get("author") {
            let authors = if let Some(authors) = authors.as_array() {
                authors
            } else {
                bail!("Expected `authors` to be an array")
            };
            let authors = authors
                .iter()
                .map(|author| {
                    let person = decode_person(author).unwrap();
                    CreativeWorkAuthors::Person(person)
                })
                .collect();
            article.authors = Some(authors);
        }

        let is_part_of = data
            .get("container-title")
            .and_then(|title| decode_string(title).ok())
            .map(|name| {
                Box::new(CreativeWorkTypes::Periodical(Periodical {
                    name: Some(Box::new(name)),
                    ..Default::default()
                }))
            });

        let is_part_of = if let Some(volume_number) = data.get("volume").and_then(|num| {
            num.as_str()
                .map(|num| PublicationVolumeVolumeNumber::String(num.to_string()))
                .or_else(|| num.as_i64().map(PublicationVolumeVolumeNumber::Integer))
        }) {
            Some(Box::new(CreativeWorkTypes::PublicationVolume(
                PublicationVolume {
                    is_part_of,
                    volume_number: Some(Box::new(volume_number)),
                    ..Default::default()
                },
            )))
        } else {
            is_part_of
        };

        let is_part_of = if let Some(issue_number) = data.get("volume").and_then(|num| {
            num.as_str()
                .map(|num| PublicationIssueIssueNumber::String(num.to_string()))
                .or_else(|| num.as_i64().map(PublicationIssueIssueNumber::Integer))
        }) {
            Some(Box::new(CreativeWorkTypes::PublicationIssue(
                PublicationIssue {
                    is_part_of,
                    issue_number: Some(Box::new(issue_number)),
                    ..Default::default()
                },
            )))
        } else {
            is_part_of
        };

        article.is_part_of = is_part_of;

        if let Some(publisher) = data.get("publisher") {
            let name = decode_string(publisher)?;
            article.publisher = Some(Box::new(CreativeWorkPublisher::Organization(
                Organization {
                    name: Some(Box::new(name)),
                    ..Default::default()
                },
            )));
        }

        Ok(Node::Article(article))
    }
}

impl CodecTrait for CslCodec {
    fn spec() -> Codec {
        Codec {
            formats: vec_string!["csl", "csl-json"],
            root_types: vec_string!["Article"],
            ..Default::default()
        }
    }

    fn from_str(str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        let data = serde_json::from_str(str)?;
        Self::from_json(data)
    }
}

/// Decode a `string` variable
fn decode_string(string: &serde_json::Value) -> Result<String> {
    string
        .as_str()
        .map(|str| str.to_string())
        .ok_or_else(|| eyre!("Expected a string value"))
}

/// Decode a `name-variable` into a `Person`
///
/// See https://github.com/citation-style-language/schema/blob/506040022f6c37846343edd36658b23e85b5b8ff/schemas/input/csl-data.json#L463
fn decode_person(data: &serde_json::Value) -> Result<Person> {
    let given_names = data
        .get("given")
        .and_then(|name| name.as_str())
        .map(|name| {
            name.split(' ')
                .map(|name| name.to_string())
                .collect::<Vec<String>>()
        });
    let family_names = data
        .get("family")
        .and_then(|name| name.as_str())
        .map(|name| {
            name.split(' ')
                .map(|name| name.to_string())
                .collect::<Vec<String>>()
        });
    let identifiers = data
        .get("ORCID")
        .or_else(|| data.get("orcid"))
        .and_then(|orcid| orcid.as_str())
        .map(|orcid| vec![ThingIdentifiers::String(orcid.to_string())]);

    Ok(Person {
        given_names,
        family_names,
        identifiers,
        ..Default::default()
    })
}

/// Decode a `date-variable` into a `Date`
///
/// See https://github.com/citation-style-language/schema/blob/506040022f6c37846343edd36658b23e85b5b8ff/schemas/input/csl-data.json#L499
fn decode_date(data: &serde_json::Value) -> Result<Date> {
    let value = if let Some(raw) = data.get("raw") {
        raw.as_str()
            .ok_or_else(|| eyre!("Expected `raw` to be a string"))?
            .to_string()
    } else if let Some(literal) = data.get("literal") {
        literal
            .as_str()
            .ok_or_else(|| eyre!("Expected `literal` to be a string"))?
            .to_string()
    } else if let Some(parts) = data.get("date-parts") {
        let parts = parts
            .as_array()
            .ok_or_else(|| eyre!("Expected `parts` to be an array"))?;
        let parts = parts
            .first()
            .and_then(|item| item.as_array())
            .ok_or_else(|| eyre!("Expected `parts` to have at least one item that is an array"))?;
        let mut date = parts
            .first()
            .and_then(|year| {
                year.as_str()
                    .map(|year| year.to_string())
                    .or_else(|| year.as_u64().map(|year| year.to_string()))
            })
            .ok_or_else(|| eyre!("Expected `year` to be a number"))?;
        if let Some(month) = parts.get(1).and_then(|month| {
            month
                .as_str()
                .map(|month| month.to_string())
                .or_else(|| month.as_u64().map(|month| month.to_string()))
        }) {
            date.push('-');
            date.push_str(&month);
        }
        if let Some(day) = parts.get(2).and_then(|day| {
            day.as_str()
                .map(|day| day.to_string())
                .or_else(|| day.as_u64().map(|day| day.to_string()))
        }) {
            date.push('-');
            date.push_str(&day);
        }
        date
    } else {
        bail!("Unable to parse `Date` from `{:?}`", data)
    };

    Ok(Date {
        value,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_snaps::{insta::assert_json_snapshot, snapshot_fixtures_content};

    #[test]
    fn fragments() {
        snapshot_fixtures_content("fragments/csl/*.json", |content| {
            let node = CslCodec::from_str(content, None).unwrap();
            assert_json_snapshot!(node);
        });
    }
}
