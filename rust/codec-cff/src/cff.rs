use serde::Deserialize;

use codec::schema::{
    Article, ArticleOptions, Author, CreativeWorkVariant, CreativeWorkVariantOrString, Date,
    IntegerOrString, Organization, Periodical, PeriodicalOptions, Person, PersonOptions,
    PersonOrOrganization, Primitive, PropertyValue, PropertyValueOrString, PublicationIssue,
    PublicationIssueOptions, PublicationVolume, PublicationVolumeOptions, SoftwareApplication,
    SoftwareApplicationOptions, SoftwareSourceCode, SoftwareSourceCodeOptions, StringOrNumber,
    shortcuts::{p, t},
};

use indexmap::IndexMap;
use itertools::Itertools;
use serde_json::Value;
use serde_with::skip_serializing_none;

/// A Citation File Format (CFF) citation file
///
/// Represents metadata for software, datasets, and other research outputs according to the
/// Citation File Format specification. CFF files are YAML documents that provide structured
/// citation information for research software and data.
///
/// See:
/// - https://citation-file-format.github.io/
/// - https://github.com/citation-file-format/citation-file-format/blob/main/schema-guide.md
/// - https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-citation-files
#[skip_serializing_none]
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CitationFile {
    /// CFF version (required)
    pub cff_version: String,

    /// Citation message (required)
    pub message: String,

    /// Title of the work (required)
    pub title: String,

    /// Authors of the work (required)
    pub authors: Vec<CffAuthor>,

    /// Type of the work (optional, defaults to software)
    #[serde(rename = "type")]
    pub work_type: Option<CffType>,

    /// Abstract or description
    pub r#abstract: Option<String>,

    /// Keywords
    pub keywords: Option<Vec<String>>,

    /// License information
    pub license: Option<String>,

    /// License URL
    pub license_url: Option<String>,

    /// Date released
    pub date_released: Option<String>,

    /// Version
    pub version: Option<String>,

    /// DOI
    pub doi: Option<String>,

    /// URL
    pub url: Option<String>,

    /// Repository URL
    pub repository: Option<String>,

    /// Repository artifact URL  
    pub repository_artifact: Option<String>,

    /// Repository code URL
    pub repository_code: Option<String>,

    /// Commit hash
    pub commit: Option<String>,

    /// Contact information
    pub contact: Option<Vec<CffAuthor>>,

    /// Identifiers
    pub identifiers: Option<Vec<CffIdentifier>>,

    /// Preferred citation (for citing related papers instead of software directly)
    pub preferred_citation: Option<PreferredCitation>,

    /// References
    pub references: Option<Vec<CffReference>>,

    /// Catch-all for other fields
    #[serde(flatten)]
    pub other: IndexMap<String, Value>,
}

/// CFF work type
///
/// Represents the type of work being cited according to the CFF specification.
/// Currently only software and dataset are supported by the specification.
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CffType {
    Software,
    Dataset,
}

/// CFF author or contact
///
/// Represents a person or entity in CFF format. Can be either a person with
/// family-names/given-names or an entity with just a name.
#[skip_serializing_none]
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CffAuthor {
    /// Family names (for persons)
    pub family_names: Option<String>,

    /// Given names (for persons)  
    pub given_names: Option<String>,

    /// Name prefix
    pub name_particle: Option<String>,

    /// Name suffix
    pub name_suffix: Option<String>,

    /// Full name (for entities or when family/given names not available)
    pub name: Option<String>,

    /// ORCID identifier
    pub orcid: Option<String>,

    /// Email address
    pub email: Option<String>,

    /// Affiliation
    pub affiliation: Option<String>,

    /// Website
    pub website: Option<String>,

    /// Other fields
    #[serde(flatten)]
    pub other: IndexMap<String, Value>,
}

/// CFF identifier
///
/// Represents an identifier for the work (DOI, URL, etc.)
#[skip_serializing_none]
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CffIdentifier {
    /// Type of identifier
    #[serde(rename = "type")]
    pub identifier_type: String,

    /// Value of identifier
    pub value: String,

    /// Description
    pub description: Option<String>,
}

/// CFF preferred citation
///
/// When present, indicates that users should cite the referenced work instead of
/// the software/dataset directly. Commonly used to cite papers about the software.
#[skip_serializing_none]
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PreferredCitation {
    /// Type of the preferred citation
    #[serde(rename = "type")]
    pub citation_type: String,

    /// Title
    pub title: String,

    /// Authors
    pub authors: Option<Vec<CffAuthor>>,

    /// DOI
    pub doi: Option<String>,

    /// URL
    pub url: Option<String>,

    /// Year
    pub year: Option<i32>,

    /// Month
    pub month: Option<i32>,

    /// Journal title
    pub journal: Option<String>,

    /// Volume
    pub volume: Option<String>,

    /// Issue
    pub issue: Option<String>,

    /// Pages (page range)
    pub pages: Option<String>,

    /// Start page
    pub start: Option<String>,

    /// End page
    pub end: Option<String>,

    /// Abstract
    pub r#abstract: Option<String>,

    /// Keywords
    pub keywords: Option<Vec<String>>,

    /// Publisher
    pub publisher: Option<Value>,

    /// Other fields
    #[serde(flatten)]
    pub other: IndexMap<String, Value>,
}

/// CFF reference
///
/// Represents a bibliographic reference in the CFF file
#[skip_serializing_none]
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CffReference {
    /// Type of reference
    #[serde(rename = "type")]
    pub reference_type: String,

    /// Title
    pub title: String,

    /// Authors
    pub authors: Option<Vec<CffAuthor>>,

    /// DOI
    pub doi: Option<String>,

    /// URL
    pub url: Option<String>,

    /// Year
    pub year: Option<i32>,

    /// Other fields
    #[serde(flatten)]
    pub other: IndexMap<String, Value>,
}

impl From<CffAuthor> for Author {
    fn from(author: CffAuthor) -> Self {
        // Check if this is an organization (has name but no family/given names)
        if let Some(ref name) = author.name
            && author.family_names.is_none()
            && author.given_names.is_none()
        {
            return Author::Organization(Organization {
                name: Some(name.clone()),
                ..Default::default()
            });
        }

        // Create a person
        let mut person = Person {
            family_names: author.family_names.map(|names| vec![names]),
            given_names: author.given_names.map(|names| vec![names]),
            orcid: author
                .orcid
                .map(|orcid| orcid.trim_start_matches("https://orcid.org/").to_string()),
            options: Box::new(PersonOptions {
                name: author.name,
                emails: author.email.map(|email| vec![email]),
                ..Default::default()
            }),
            ..Default::default()
        };

        // Add other identifiers (not ORCID)
        let mut identifiers = Vec::new();
        if let Some(website) = author.website {
            identifiers.push(PropertyValueOrString::String(website));
        }

        if !identifiers.is_empty() {
            person.options.identifiers = Some(identifiers);
        }

        Author::Person(person)
    }
}

impl From<PreferredCitation> for Article {
    fn from(citation: PreferredCitation) -> Self {
        let title = vec![t(&citation.title)];

        let authors = citation
            .authors
            .map(|authors| authors.into_iter().map(Author::from).collect_vec())
            .filter(|authors| !authors.is_empty());

        // Create date from year and month
        let date_published = citation.year.map(|year| {
            if let Some(month) = citation.month {
                Date::new(format!("{year}-{month:02}"))
            } else {
                Date::new(year.to_string())
            }
        });

        let r#abstract = citation.r#abstract.map(|text| vec![p(vec![t(text)])]);

        let doi = citation
            .doi
            .map(|doi| doi.trim_start_matches("https://doi.org/").to_string());

        let _keywords = citation.keywords;

        let publisher = citation.publisher.and_then(|publisher_value| {
            match publisher_value {
                Value::String(name) => Some(PersonOrOrganization::Organization(Organization {
                    name: Some(name),
                    ..Default::default()
                })),
                Value::Object(ref map) => {
                    // Try to extract name from object
                    map.get("name").and_then(|v| v.as_str()).map(|name| {
                        PersonOrOrganization::Organization(Organization {
                            name: Some(name.to_string()),
                            ..Default::default()
                        })
                    })
                }
                _ => None,
            }
        });

        // Handle journal information with volume, issue, and pages
        let is_part_of = citation.journal.as_ref().map(|journal_name| {
            create_publication_info(
                journal_name,
                citation.volume.as_ref(),
                citation.issue.as_ref(),
                citation.pages.as_deref().or(citation.start.as_deref()),
                citation.start.as_deref(),
                citation.end.as_deref(),
            )
        });

        Article {
            title: Some(title),
            authors,
            r#abstract,
            doi,
            date_published,
            options: Box::new(ArticleOptions {
                url: citation.url,
                publisher,
                is_part_of,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

impl From<CitationFile> for SoftwareSourceCode {
    fn from(cff: CitationFile) -> Self {
        let name = cff.title;

        let authors = cff.authors.into_iter().map(Author::from).collect_vec();
        let authors = (!authors.is_empty()).then_some(authors);

        let description = cff.r#abstract;

        let date_published = cff.date_released.map(Date::new);

        let programming_language = String::new(); // CFF doesn't specify this

        let licenses = cff
            .license
            .or(cff.license_url)
            .map(|license| vec![CreativeWorkVariantOrString::String(license)]);

        let mut identifiers = Vec::new();
        if let Some(ids) = cff.identifiers {
            for id in ids {
                identifiers.push(PropertyValueOrString::PropertyValue(PropertyValue {
                    property_id: Some(id.identifier_type),
                    value: Primitive::String(id.value),
                    ..Default::default()
                }));
            }
        }
        let identifiers = (!identifiers.is_empty()).then_some(identifiers);

        SoftwareSourceCode {
            name,
            programming_language,
            repository: cff.repository_code.or(cff.repository),
            version: cff.version.map(StringOrNumber::String),
            doi: cff.doi,
            options: Box::new(SoftwareSourceCodeOptions {
                description,
                authors,
                date_published,
                licenses,
                identifiers,
                keywords: cff.keywords,
                url: cff.url,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

impl From<CitationFile> for SoftwareApplication {
    fn from(cff: CitationFile) -> Self {
        let name = cff.title;

        let authors = cff.authors.into_iter().map(Author::from).collect_vec();
        let authors = (!authors.is_empty()).then_some(authors);

        let description = cff.r#abstract;

        let date_published = cff.date_released.map(Date::new);

        let licenses = cff
            .license
            .or(cff.license_url)
            .map(|license| vec![CreativeWorkVariantOrString::String(license)]);

        let mut identifiers = Vec::new();
        if let Some(ids) = cff.identifiers {
            for id in ids {
                identifiers.push(PropertyValueOrString::PropertyValue(PropertyValue {
                    property_id: Some(id.identifier_type),
                    value: Primitive::String(id.value),
                    ..Default::default()
                }));
            }
        }
        let identifiers = (!identifiers.is_empty()).then_some(identifiers);

        SoftwareApplication {
            name,
            version: cff.version.map(StringOrNumber::String),
            doi: cff.doi,
            options: Box::new(SoftwareApplicationOptions {
                description,
                authors,
                date_published,
                licenses,
                identifiers,
                keywords: cff.keywords,
                url: cff.url,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

/// Create publication hierarchy from CFF preferred citation metadata
///
/// Similar to CSL's create_publication_info but adapted for CFF structure.
/// Creates a hierarchical publication structure: Periodical -> Volume -> Issue
/// with page information at the appropriate level.
fn create_publication_info(
    journal_name: &str,
    volume: Option<&String>,
    issue: Option<&String>,
    page_range: Option<&str>,
    start_page: Option<&str>,
    end_page: Option<&str>,
) -> CreativeWorkVariant {
    let periodical = Periodical {
        name: Some(journal_name.to_string()),
        options: Box::new(PeriodicalOptions {
            ..Default::default()
        }),
        ..Default::default()
    };

    if let Some(vol) = volume {
        let mut publication_volume = PublicationVolume {
            is_part_of: Some(Box::new(CreativeWorkVariant::Periodical(periodical))),
            volume_number: Some(IntegerOrString::String(vol.clone())),
            options: Box::new(PublicationVolumeOptions {
                ..Default::default()
            }),
            ..Default::default()
        };

        if let Some(iss) = issue {
            let mut publication_issue = PublicationIssue {
                is_part_of: Some(Box::new(CreativeWorkVariant::PublicationVolume(
                    publication_volume,
                ))),
                issue_number: Some(IntegerOrString::String(iss.clone())),
                options: Box::new(PublicationIssueOptions {
                    ..Default::default()
                }),
                ..Default::default()
            };

            // Add page information to the issue level
            if let Some(page_range) = page_range {
                publication_issue.options.page_start = extract_page_start(page_range);
                publication_issue.options.page_end = extract_page_end(page_range);
            } else {
                // Use explicit start/end pages if no range provided
                publication_issue.options.page_start =
                    start_page.map(|p| IntegerOrString::String(p.to_string()));
                publication_issue.options.page_end =
                    end_page.map(|p| IntegerOrString::String(p.to_string()));
            }

            CreativeWorkVariant::PublicationIssue(publication_issue)
        } else {
            // Add page information to the volume level if no issue
            if let Some(page_range) = page_range {
                publication_volume.options.page_start = extract_page_start(page_range);
                publication_volume.options.page_end = extract_page_end(page_range);
            } else {
                // Use explicit start/end pages if no range provided
                publication_volume.options.page_start =
                    start_page.map(|p| IntegerOrString::String(p.to_string()));
                publication_volume.options.page_end =
                    end_page.map(|p| IntegerOrString::String(p.to_string()));
            }

            CreativeWorkVariant::PublicationVolume(publication_volume)
        }
    } else {
        CreativeWorkVariant::Periodical(periodical)
    }
}

/// Extract page start from page range (e.g., "123-456" -> Some("123"))
///
/// Adapted from CSL codec for consistency in page range parsing.
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
///
/// Adapted from CSL codec for consistency in page range parsing.
fn extract_page_end(page_range: &str) -> Option<IntegerOrString> {
    if let Some(dash_pos) = page_range.find('-') {
        let end = page_range[dash_pos + 1..].trim();
        if !end.is_empty() {
            return Some(IntegerOrString::String(end.to_string()));
        }
    }
    None
}
