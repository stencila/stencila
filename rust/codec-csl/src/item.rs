use codec::{
    common::{
        indexmap::IndexMap,
        serde::{Deserialize, Serialize},
        serde_json::Value,
        serde_with::skip_serializing_none,
    },
    schema::{
        Article, ArticleOptions, CreativeWorkType, IntegerOrString, Organization, Periodical,
        PersonOrOrganization, PublicationIssue, PublicationVolume,
        shortcuts::{p, t},
    },
};

use crate::{
    date::{DateField, convert_csl_date},
    name::{NameField, convert_csl_authors},
    ordinary::OrdinaryField,
};

/// A CSL item
///
/// Represents a bibliographic item in Citation Style Language JSON format.
/// This implementation aims to be comprehensive and flexible, supporting both
/// standard CSL fields and extension fields used by various publishers and repositories.
///
/// See:
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
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", crate = "codec::common::serde")]
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
    pub doi: Option<String>,
    pub url: Option<String>,
    pub issn: Option<Vec<String>>,
    pub isbn: Option<Vec<String>>,
    pub pmid: Option<String>,
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
/// - https://citeproc-js.readthedocs.io/en/latest/csl-json/markup.html#introduction
/// - https://docs.citationstyles.org/en/stable/specification.html#appendix-iii-types
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", crate = "codec::common::serde")]
pub enum ItemType {
    #[default]
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
    JournalArticle,
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
    PersonalCommunication,
    Post,
    PostWeblog,
    PostedContent,
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

impl From<Item> for Article {
    fn from(item: Item) -> Self {
        let title = item.title.as_ref().map(|title_str| vec![t(title_str)]);

        let authors = item
            .author
            .as_ref()
            .map(|authors| convert_csl_authors(authors))
            .unwrap_or_default();

        let authors = (!authors.is_empty()).then_some(authors);

        let date_published = item.issued.as_ref().and_then(convert_csl_date);

        let r#abstract = item
            .abstract_text
            .as_ref()
            .map(|text| vec![p(vec![t(text)])]);

        let doi = item.doi.clone();
        let url = item.url.clone();

        let is_part_of = item.container_title.as_ref().and_then(|container| {
            create_publication_info(
                container,
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

        Article {
            title,
            authors,
            r#abstract,
            doi,
            date_published,
            options: Box::new(ArticleOptions {
                url,
                is_part_of,
                publisher,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

/// Create publication hierarchy from CSL metadata
fn create_publication_info(
    container_title: &Value,
    volume: Option<&OrdinaryField>,
    issue: Option<&OrdinaryField>,
    page: Option<&str>,
) -> Option<CreativeWorkType> {
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
        ..Default::default()
    };

    if let Some(vol) = volume {
        let volume_str = match vol {
            OrdinaryField::String(s) => s.clone(),
            OrdinaryField::Integer(n) => n.to_string(),
            OrdinaryField::Float(n) => n.to_string(),
        };

        let mut publication_volume = PublicationVolume {
            is_part_of: Some(Box::new(CreativeWorkType::Periodical(periodical))),
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
                is_part_of: Some(Box::new(CreativeWorkType::PublicationVolume(
                    publication_volume,
                ))),
                issue_number: Some(IntegerOrString::String(issue_str)),
                ..Default::default()
            };

            if let Some(page_range) = page {
                publication_issue.options.page_start = extract_page_start(page_range);
                publication_issue.options.page_end = extract_page_end(page_range);
            }

            Some(CreativeWorkType::PublicationIssue(publication_issue))
        } else {
            if let Some(page_range) = page {
                publication_volume.options.page_start = extract_page_start(page_range);
                publication_volume.options.page_end = extract_page_end(page_range);
            }
            Some(CreativeWorkType::PublicationVolume(publication_volume))
        }
    } else {
        Some(CreativeWorkType::Periodical(periodical))
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
