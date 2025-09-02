use serde::Deserialize;

use stencila_codec::stencila_schema::{
    self, Article, ArticleOptions, Author, Block, CreativeWorkVariant, Date, IntegerOrString,
    Organization, Paragraph, Periodical, PeriodicalOptions, PersonOrOrganization, Primitive,
    PropertyValue, PropertyValueOrString, PublicationIssue, PublicationVolume, Reference,
    shortcuts::{p, t},
};

use indexmap::IndexMap;
use itertools::Itertools;
use serde_json::Value;
use serde_with::skip_serializing_none;

use crate::{date::DateField, name::NameField, ordinary::OrdinaryField};

/// A CSL item
///
/// Represents a bibliographic item in Citation Style Language JSON format.
/// This implementation aims to be comprehensive and flexible, supporting both
/// standard CSL fields and extension fields used by various publishers and repositories.
///
/// See:
/// - https://docs.citationstyles.org/en/stable/specification.html#appendix-iii-types
/// - https://citeproc-js.readthedocs.io/en/latest/csl-json/markup.html#items
/// - https://raw.githubusercontent.com/citation-style-language/schema/master/schemas/input/csl-data.json
///
/// The `citeworks_csl` crate also provides similar Rust types but, at the time
/// of writing, had not be maintained for several years.
///
/// Given that, and to avoid adding another dependency, this crate implements
/// serde deserialization and serialization for CSL types and implements the
/// `From` trait to convert CSL types to Stencila Schema types.
#[skip_serializing_none]
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Item {
    /// Unique identifier
    pub id: Option<String>,

    /// Type of the item
    #[serde(rename = "type")]
    pub item_type: ItemType,

    // Title fields
    pub title: Option<String>,
    pub title_short: Option<String>,
    pub subtitle: Option<Vec<String>>,
    pub short_title: Option<Vec<String>>,

    // Creator fields
    pub author: Option<Vec<NameField>>,
    pub editor: Option<Vec<NameField>>,
    pub translator: Option<Vec<NameField>>,
    pub recipient: Option<Vec<NameField>>,
    pub interviewer: Option<Vec<NameField>>,
    pub composer: Option<Vec<NameField>>,
    pub director: Option<Vec<NameField>>,
    pub illustrator: Option<Vec<NameField>>,

    // Date fields
    pub issued: Option<DateField>,
    pub submitted: Option<DateField>,
    pub accessed: Option<DateField>,
    pub event_date: Option<DateField>,
    pub original_date: Option<DateField>,

    // Publication fields
    pub container_title: Option<Value>,
    pub container_title_short: Option<String>,
    pub collection_title: Option<String>,
    pub collection_number: Option<OrdinaryField>,
    pub volume: Option<OrdinaryField>,
    pub issue: Option<OrdinaryField>,
    pub edition: Option<OrdinaryField>,
    pub page: Option<String>,
    pub page_first: Option<String>,
    pub number_of_pages: Option<OrdinaryField>,
    pub chapter_number: Option<OrdinaryField>,

    // Identifier fields
    #[serde(alias = "DOI")]
    pub doi: Option<String>,
    #[serde(alias = "URL")]
    pub url: Option<String>,
    #[serde(alias = "ISSN")]
    pub issn: Option<Vec<String>>,
    #[serde(alias = "ISBN")]
    pub isbn: Option<Vec<String>>,
    #[serde(alias = "PMID")]
    pub pmid: Option<String>,
    #[serde(alias = "PMCID")]
    pub pmcid: Option<String>,

    // Publication info
    pub publisher: Option<String>,
    pub publisher_place: Option<String>,
    pub jurisdiction: Option<String>,

    // Description fields
    #[serde(rename = "abstract")]
    pub abstract_text: Option<String>,
    pub note: Option<String>,
    pub annote: Option<String>,
    pub keyword: Option<String>,

    // Classification fields
    pub genre: Option<String>,
    pub medium: Option<String>,
    pub status: Option<String>,
    pub version: Option<String>,

    // Event fields
    pub event: Option<String>,
    pub event_place: Option<String>,

    // Archive fields
    pub archive: Option<String>,
    pub archive_location: Option<String>,
    pub archive_place: Option<String>,
    pub call_number: Option<String>,

    // Technical fields
    pub language: Option<String>,
    pub source: Option<String>,
    pub references: Option<String>,
    pub dimensions: Option<String>,
    pub scale: Option<String>,

    // Legal fields
    pub authority: Option<String>,
    pub section: Option<String>,

    // Numbers and counts
    pub citation_number: Option<OrdinaryField>,
    pub citation_label: Option<String>,
    pub first_reference_note_number: Option<OrdinaryField>,
    pub locator: Option<String>,
    pub label: Option<String>,

    // Bibliographic metadata
    pub reference_count: Option<OrdinaryField>,
    pub references_count: Option<OrdinaryField>,
    pub is_referenced_by_count: Option<OrdinaryField>,

    // Repository/Database metadata
    pub indexed: Option<DateField>,
    pub deposited: Option<DateField>,
    pub posted: Option<DateField>,
    pub accepted: Option<DateField>,
    pub published_print: Option<DateField>,
    pub published_online: Option<DateField>,

    // Classification and categorization
    pub subject: Option<Vec<String>>,
    pub funder: Option<Vec<Value>>,
    pub license: Option<Vec<Value>>,
    pub link: Option<Vec<Value>>,
    pub relation: Option<Value>,

    // Publication structure
    pub journal_issue: Option<Value>,
    pub original_title: Option<Vec<String>>,
    pub alternative_id: Option<Vec<String>>,

    // Content metadata
    pub content_domain: Option<Value>,
    pub subtype: Option<String>,
    pub categories: Option<Vec<String>>,
    pub group_title: Option<String>,
    pub institution: Option<Vec<Value>>,

    // Technical metadata
    pub score: Option<f64>,
    pub resource: Option<Value>,
    pub prefix: Option<String>,
    pub member: Option<String>,

    // Reference list
    pub reference: Option<Vec<ReferenceValue>>,

    // Catch-all for other fields
    // Uses `IndexMap` so that order is deterministic
    #[serde(flatten)]
    pub other: IndexMap<String, Value>,
}

/// A CSL item type
///
/// Represents the type of bibliographic resource according to the CSL specification.
/// This enum includes all standard CSL item types plus common extensions.
///
/// See:
/// - https://docs.citationstyles.org/en/stable/specification.html#appendix-iii-types
/// - https://docs.citationstyles.org/en/stable/specification.html#appendix-iv-variables
/// - https://citeproc-js.readthedocs.io/en/latest/csl-json/markup.html#introduction
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ItemType {
    Article,
    ArticleJournal,
    ArticleMagazine,
    ArticleNewspaper,
    Bill,
    Book,
    Broadcast,
    Chapter,
    Classic,
    Collection,
    Dataset,
    Document,
    Entry,
    EntryDictionary,
    EntryEncyclopedia,
    Event,
    Figure,
    Graphic,
    Hearing,
    Interview,
    LegalCase,
    Legislation,
    Manuscript,
    Map,
    MotionPicture,
    MusicalScore,
    Pamphlet,
    PaperConference,
    Patent,
    Performance,
    Periodical,
    PersonalCommunication,
    Post,
    PostWeblog,
    Regulation,
    Report,
    Review,
    ReviewBook,
    Software,
    Song,
    Speech,
    Standard,
    Thesis,
    Treaty,
    Webpage,
    #[serde(other)]
    Other,
}

/// A CSL reference value
///
/// Represents a bibliographic reference in CSL-JSON format. References can contain
/// structured metadata or unstructured citation strings, and field names vary
/// between publishers (camelCase vs kebab-case).
///
/// See:
/// - https://docs.citationstyles.org/en/stable/specification.html#appendix-iv-variables
/// - https://www.crossref.org/documentation/retrieve-metadata/rest-api/
#[skip_serializing_none]
#[derive(Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ReferenceValue {
    /// Unique key for this reference
    pub key: Option<String>,

    /// DOI of the referenced work
    #[serde(alias = "DOI")]
    pub doi: Option<String>,

    /// Author information (often as a simple string)
    pub author: Option<String>,

    /// Publication year
    pub year: Option<String>,

    /// Title of the referenced article
    pub article_title: Option<String>,

    /// Journal or publication title
    pub journal_title: Option<String>,

    /// Volume title (for books, conference proceedings)
    pub volume_title: Option<String>,

    /// Volume number
    pub volume: Option<String>,

    /// Issue number
    pub issue: Option<String>,

    /// First page number
    pub first_page: Option<String>,

    /// Publisher name
    pub publisher: Option<String>,

    /// Unstructured citation string (fallback)
    pub unstructured: Option<String>,

    /// DOI assertion information
    pub doi_asserted_by: Option<String>,

    /// Catch-all for other fields
    #[serde(flatten)]
    pub other: IndexMap<String, Value>,
}

impl From<ReferenceValue> for Reference {
    fn from(ref_value: ReferenceValue) -> Self {
        // Clean DOI by removing URL prefix if present
        let doi = ref_value
            .doi
            .map(|doi| doi.trim_start_matches("https://doi.org/").to_string());

        // Parse authors from string - for now just create a single author if present
        let authors = ref_value.author.and_then(|author_str| {
            if author_str.trim().is_empty() {
                None
            } else {
                // Try to parse as a name, fallback to literal
                use std::str::FromStr;
                use stencila_schema::{Person, PersonOptions};

                let person = Person::from_str(&author_str).unwrap_or_else(|_| Person {
                    options: Box::new(PersonOptions {
                        name: Some(author_str),
                        ..Default::default()
                    }),
                    ..Default::default()
                });
                Some(vec![stencila_schema::Author::Person(person)])
            }
        });

        // Parse year into Date
        let date = ref_value.year.map(Date::new);

        // Create title from available title fields
        let title = ref_value
            .article_title
            .or(ref_value.unstructured)
            .or(ref_value.volume_title.clone())
            .map(|title_str| vec![t(&title_str)]);

        // Create is_part_of if journal info is available
        let is_part_of = ref_value
            .journal_title
            .map(|journal_title| Reference {
                work_type: Some(stencila_schema::CreativeWorkType::Periodical),
                title: Some(vec![t(&journal_title)]),
                issue_number: ref_value.issue.map(IntegerOrString::String),
                volume_number: ref_value.volume.map(IntegerOrString::String),
                ..Default::default()
            })
            .map(Box::new);

        // Extract page start from first-page
        let page_start = ref_value.first_page.map(IntegerOrString::String);

        Reference {
            doi,
            authors,
            date,
            title,
            is_part_of,
            page_start,
            ..Default::default()
        }
    }
}

impl From<Item> for Article {
    fn from(item: Item) -> Self {
        let title = item.title.as_ref().map(|title_str| vec![t(title_str)]);

        let authors = item
            .author
            .into_iter()
            .flatten()
            .map(Author::from)
            .collect_vec();
        let authors = (!authors.is_empty()).then_some(authors);

        let date_published = item.issued.and_then(|date| Date::try_from(date).ok());

        let r#abstract = item.abstract_text.map(parse_jats_paragraphs);

        let url = item.url.clone();

        let doi = item
            .doi
            .clone()
            .map(|doi| doi.trim_start_matches("https://doi.org").to_string());

        let mut identifiers = Vec::new();
        if let Some(id) = item.pmid {
            identifiers.push(id_to_identifier("pmid", id));
        }
        if let Some(id) = item.pmcid {
            identifiers.push(id_to_identifier("pmcid", id));
        }
        let identifiers = (!identifiers.is_empty()).then_some(identifiers);

        let is_part_of = item.container_title.as_ref().and_then(|container| {
            create_publication_info(
                container,
                item.issn,
                item.volume.as_ref(),
                item.issue.as_ref(),
                item.page.as_deref(),
            )
        });

        let publisher = item.publisher.map(|publisher| {
            PersonOrOrganization::Organization(Organization {
                name: Some(publisher),
                ..Default::default()
            })
        });

        let references = item
            .reference
            .map(|refs| refs.into_iter().map(Reference::from).collect_vec())
            .filter(|refs| !refs.is_empty());

        Article {
            title,
            authors,
            r#abstract,
            doi,
            date_published,
            references,
            options: Box::new(ArticleOptions {
                url,
                identifiers,
                is_part_of,
                publisher,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

/// Parse JATS paragraphs from abstract text
///
/// Extracts separate paragraphs between <jats:p> and </jats:p> tags, dropping
/// those tags where appropriate. If those tags are not in the text then just
/// use a single paragraph.
fn parse_jats_paragraphs(text: String) -> Vec<Block> {
    // Check if the text contains JATS paragraph tags
    if text.contains("<jats:p>") && text.contains("</jats:p>") {
        // Split by JATS paragraph tags and extract content
        let paragraphs: Vec<_> = text
            .split("<jats:p>")
            .skip(1) // Skip the part before the first <jats:p>
            .filter_map(|part| {
                // Find the closing tag and extract content
                part.split_once("</jats:p>")
                    .map(|(content, _)| content.trim())
                    .filter(|content| !content.is_empty())
                    .map(|content| {
                        Block::Paragraph(Paragraph {
                            content: vec![t(content)],
                            ..Default::default()
                        })
                    })
            })
            .collect();
        if !paragraphs.is_empty() {
            return paragraphs;
        }
    }

    // No JATS tags found, use single paragraph
    vec![p(vec![t(text)])]
}

/// Convert a string id to valid `identifiers` value
fn id_to_identifier(property_id: &str, id: String) -> PropertyValueOrString {
    // If the value is a URL, use it directly as a string identifier
    if id.starts_with("http://") || id.starts_with("https://") {
        PropertyValueOrString::String(id)
    } else {
        // Otherwise create a PropertyValue with property_id and value
        PropertyValueOrString::PropertyValue(PropertyValue {
            property_id: Some(property_id.to_string()),
            value: Primitive::String(id),
            ..Default::default()
        })
    }
}

/// Create publication hierarchy from CSL metadata
fn create_publication_info(
    container_title: &Value,
    issns: Option<Vec<String>>,
    volume: Option<&OrdinaryField>,
    issue: Option<&OrdinaryField>,
    page: Option<&str>,
) -> Option<CreativeWorkVariant> {
    let periodical_name = match container_title {
        Value::String(value) => value.to_string(),
        Value::Array(value) => value
            .first()
            .map(|value| value.to_string())
            .unwrap_or_default(),
        _ => container_title.to_string(),
    };

    let periodical = Periodical {
        name: Some(periodical_name),
        options: Box::new(PeriodicalOptions {
            issns,
            ..Default::default()
        }),
        ..Default::default()
    };

    if let Some(vol) = volume {
        let volume_str = match vol {
            OrdinaryField::String(s) => s.clone(),
            OrdinaryField::Integer(n) => n.to_string(),
            OrdinaryField::Float(n) => n.to_string(),
        };

        let mut publication_volume = PublicationVolume {
            is_part_of: Some(Box::new(CreativeWorkVariant::Periodical(periodical))),
            volume_number: Some(IntegerOrString::String(volume_str)),
            ..Default::default()
        };

        if let Some(iss) = issue {
            let issue_str = match iss {
                OrdinaryField::String(s) => s.clone(),
                OrdinaryField::Integer(n) => n.to_string(),
                OrdinaryField::Float(n) => n.to_string(),
            };

            let mut publication_issue = PublicationIssue {
                is_part_of: Some(Box::new(CreativeWorkVariant::PublicationVolume(
                    publication_volume,
                ))),
                issue_number: Some(IntegerOrString::String(issue_str)),
                ..Default::default()
            };

            if let Some(page_range) = page {
                publication_issue.options.page_start = extract_page_start(page_range);
                publication_issue.options.page_end = extract_page_end(page_range);
            }

            Some(CreativeWorkVariant::PublicationIssue(publication_issue))
        } else {
            if let Some(page_range) = page {
                publication_volume.options.page_start = extract_page_start(page_range);
                publication_volume.options.page_end = extract_page_end(page_range);
            }
            Some(CreativeWorkVariant::PublicationVolume(publication_volume))
        }
    } else {
        Some(CreativeWorkVariant::Periodical(periodical))
    }
}

/// Extract page start from page range (e.g., "123-456" -> Some("123"))
fn extract_page_start(page_range: &str) -> Option<IntegerOrString> {
    if let Some(dash_pos) = page_range.find('-') {
        let start = page_range[..dash_pos].trim();
        if !start.is_empty() {
            return Some(IntegerOrString::String(start.to_string()));
        }
    }
    // Single page or no dash
    let trimmed = page_range.trim();
    if !trimmed.is_empty() {
        Some(IntegerOrString::String(trimmed.to_string()))
    } else {
        None
    }
}

/// Extract page end from page range (e.g., "123-456" -> Some("456"))
fn extract_page_end(page_range: &str) -> Option<IntegerOrString> {
    if let Some(dash_pos) = page_range.find('-') {
        let end = page_range[dash_pos + 1..].trim();
        if !end.is_empty() {
            return Some(IntegerOrString::String(end.to_string()));
        }
    }
    None
}
