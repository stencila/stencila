use std::collections::HashMap;

use indexmap::IndexMap;
use itertools::Itertools;
use serde::Deserialize;

use stencila_codec::{
    eyre::{Result, bail},
    stencila_schema::{
        self, Article, ArticleOptions, Block, CreativeWork, CreativeWorkOptions, CreativeWorkType,
        CreativeWorkVariant, CreativeWorkVariantOrString, Date, Inline, IntegerOrString, Node,
        Organization, OrganizationOptions, Paragraph, Periodical, Person, PropertyValueOrString,
        PublicationIssue, PublicationVolume, Reference, ReferenceOptions,
    },
};

use crate::{
    license::normalize_license,
    utils::{convert_ids_to_identifiers, strip_doi_prefix, strip_orcid_prefix, strip_ror_prefix},
};

/// An OpenAlex `Work` object
///
/// See https://docs.openalex.org/api-entities/works/work-object
#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Work {
    pub id: String,
    pub display_name: Option<String>,
    pub title: Option<String>,
    pub doi: Option<String>,
    pub ids: Option<IndexMap<String, String>>,
    pub publication_date: Option<String>,
    pub publication_year: Option<i32>,
    pub language: Option<String>,
    pub r#type: Option<String>,
    pub type_crossref: Option<String>,
    pub open_access: Option<OpenAccess>,
    pub authorships: Option<Vec<Authorship>>,
    pub abstract_inverted_index: Option<HashMap<String, Vec<i32>>>,
    pub cited_by_count: Option<i64>,
    pub biblio: Option<Biblio>,
    pub is_retracted: Option<bool>,
    pub is_paratext: Option<bool>,
    pub primary_location: Option<Location>,
    pub locations: Option<Vec<Location>>,
    pub best_oa_location: Option<Location>,
    pub sustainable_development_goals: Option<Vec<SustainableDevelopmentGoal>>,
    pub grants: Option<Vec<Grant>>,
    pub datasets: Option<Vec<String>>,
    pub versions: Option<Vec<String>>,
    pub referenced_works: Option<Vec<String>>,
    pub related_works: Option<Vec<String>>,
    pub ngrams_url: Option<String>,
    pub abstract_inverted_index_url: Option<String>,
    pub cited_by_api_url: Option<String>,
    pub counts_by_year: Option<Vec<CountsByYear>>,
    pub updated_date: Option<String>,
    pub created_date: Option<String>,

    /// The fetched `referenced_works`
    #[serde(skip)]
    pub referenced_works_fetched: Vec<Work>,
}

impl Work {
    /// Get the DOI of a work, or generate a pseudo DOI
    pub fn doi(&self) -> String {
        if let Some(doi) = &self.doi {
            doi.trim_start_matches("https://doi.org/").into()
        } else {
            let id = self.id.trim_start_matches("https://openalex.org/");
            format!("10.0000/openalex.{id}")
        }
    }
}

/// An OpenAlex `Authorship` object
///
/// See https://docs.openalex.org/api-entities/works/work-object/authorship-object
#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Authorship {
    pub author_position: Option<String>,
    pub author: Option<DehydratedAuthor>,
    pub institutions: Option<Vec<DehydratedInstitution>>,
    pub countries: Option<Vec<String>>,
    pub is_corresponding: Option<bool>,
    pub raw_author_name: Option<String>,
    pub raw_affiliation_strings: Option<Vec<String>>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct DehydratedAuthor {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub orcid: Option<String>,
}

impl DehydratedAuthor {
    /// Get the ORCID of an author, or generate a pseudo ORCID
    pub fn orcid(&self, prefix: char) -> Result<String> {
        if let Some(id) = &self.id {
            crate::utils::get_or_generate_orcid(&self.orcid, id, prefix)
        } else {
            bail!("Missing author ID")
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct DehydratedInstitution {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub ror: Option<String>,
    pub country_code: Option<String>,
    pub r#type: Option<String>,
    pub lineage: Option<Vec<String>>,
}

impl DehydratedInstitution {
    /// Get the ROR of an institution, or generate a pseudo ROR
    pub fn ror(&self, prefix: char) -> String {
        if let Some(id) = &self.id {
            crate::utils::get_or_generate_ror(&self.ror, id, prefix)
        } else {
            format!("{prefix}unknown")
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct OpenAccess {
    pub is_oa: Option<bool>,
    pub oa_date: Option<String>,
    pub oa_url: Option<String>,
    pub any_repository_has_fulltext: Option<bool>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Biblio {
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub first_page: Option<String>,
    pub last_page: Option<String>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Location {
    pub source: Option<DehydratedSource>,
    pub landing_page_url: Option<String>,
    pub pdf_url: Option<String>,
    pub is_oa: Option<bool>,
    pub version: Option<String>,
    pub license: Option<String>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct DehydratedSource {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub issn_l: Option<String>,
    pub issn: Option<Vec<String>>,
    pub is_oa: Option<bool>,
    pub is_in_doaj: Option<bool>,
    pub is_core: Option<bool>,
    pub host_organization: Option<String>,
    pub host_organization_name: Option<String>,
    pub host_organization_lineage: Option<Vec<String>>,
    pub r#type: Option<String>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct SustainableDevelopmentGoal {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub score: Option<f64>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Grant {
    pub funder: Option<String>,
    pub funder_display_name: Option<String>,
    pub award_id: Option<String>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct CountsByYear {
    pub year: Option<i32>,
    pub cited_by_count: Option<i64>,
}

impl From<Work> for Article {
    fn from(work: Work) -> Self {
        let title = extract_title(&work);
        let doi = extract_doi(&work);
        let r#abstract = extract_abstract(&work);
        let identifiers = extract_identifiers(&work);
        let date_published = extract_publication_date(&work);
        let authors = extract_authors(&work);
        let (page_start, page_end) = extract_page_info(&work);
        let is_part_of = extract_publication_info(&work).map(|info| *info);
        let licenses = extract_licenses(&work);

        let references = work
            .referenced_works_fetched
            .into_iter()
            .map(Into::into)
            .collect_vec();
        let references = (!references.is_empty()).then_some(references);

        Article {
            doi,
            title,
            r#abstract,
            date_published,
            authors,
            references,
            options: Box::new(ArticleOptions {
                identifiers,
                page_start,
                page_end,
                is_part_of,
                licenses,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

impl From<Work> for Reference {
    fn from(work: Work) -> Self {
        let work_type = map_work_type(work.r#type.as_ref());
        let title = extract_title(&work);
        let doi = extract_doi(&work);
        let identifiers = extract_identifiers(&work);
        let date = extract_publication_date(&work);
        let authors = extract_authors(&work);
        let is_part_of = create_reference_part_of(&work);
        let (page_start, page_end) = extract_page_info(&work);
        let (volume_number, issue_number) = extract_volume_issue(&work);

        Reference {
            work_type,
            doi,
            authors,
            title,
            date,
            is_part_of,
            options: Box::new(ReferenceOptions {
                editors: None,
                publisher: None,
                volume_number,
                issue_number,
                page_start,
                page_end,
                pagination: None,
                identifiers,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

impl From<Work> for CreativeWork {
    fn from(work: Work) -> Self {
        let title = extract_title(&work);
        let doi = extract_doi(&work);
        let r#abstract = extract_abstract(&work);
        let identifiers = extract_identifiers(&work);
        let date_published = extract_publication_date(&work);
        let authors = extract_authors(&work);
        let is_part_of = extract_publication_info(&work).map(|info| *info);
        let licenses = extract_licenses(&work);

        CreativeWork {
            doi,
            options: Box::new(CreativeWorkOptions {
                title,
                r#abstract,
                identifiers,
                date_published,
                authors,
                is_part_of,
                licenses,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

impl From<Work> for Node {
    fn from(work: Work) -> Self {
        if matches!(work.r#type.as_deref(), Some("article")) {
            Node::Article(work.into())
        } else {
            Node::CreativeWork(work.into())
        }
    }
}

/// Extract title from a Work
fn extract_title(work: &Work) -> Option<Vec<Inline>> {
    work.display_name
        .clone()
        .or(work.title.clone())
        .map(|title| vec![Inline::Text(title.into())])
}

/// Extract DOI from a Work, including fallback to URLs
fn extract_doi(work: &Work) -> Option<String> {
    let mut doi = work.doi.clone();
    if doi.is_none() {
        doi = work
            .primary_location
            .as_ref()
            .and_then(|location| location.landing_page_url.as_ref())
            .and_then(|url| url.strip_prefix("https://doi.org/").map(String::from));
    }
    if doi.is_none() {
        doi = work
            .open_access
            .as_ref()
            .and_then(|location| location.oa_url.as_ref())
            .and_then(|url| url.strip_prefix("https://doi.org/").map(String::from));
    }
    strip_doi_prefix(doi)
}

/// Extract identifiers from a Work
fn extract_identifiers(work: &Work) -> Option<Vec<PropertyValueOrString>> {
    work.ids.as_ref().and_then(convert_ids_to_identifiers)
}

/// Extract publication date from a Work
fn extract_publication_date(work: &Work) -> Option<Date> {
    work.publication_date.clone().map(Date::new)
}

/// Extract authors from a Work
fn extract_authors(work: &Work) -> Option<Vec<stencila_schema::Author>> {
    work.authorships.as_ref().and_then(|authorships| {
        let authors: Vec<stencila_schema::Author> = authorships
            .iter()
            .filter_map(|authorship| {
                authorship.author.as_ref().map(|dehydrated_author| {
                    let affiliations = authorship
                        .institutions
                        .as_ref()
                        .map(|institutions| {
                            institutions
                                .iter()
                                .map(|inst| Organization {
                                    name: inst.display_name.clone(),
                                    ror: strip_ror_prefix(inst.ror.clone()),
                                    options: Box::new(OrganizationOptions::default()),
                                    ..Default::default()
                                })
                                .collect()
                        })
                        .filter(|orgs: &Vec<Organization>| !orgs.is_empty());

                    let mut person = match &dehydrated_author.display_name {
                        Some(name) => Person::from(name),
                        None => Person::default(),
                    };
                    person.orcid = strip_orcid_prefix(dehydrated_author.orcid.clone());
                    person.affiliations = affiliations;

                    stencila_schema::Author::Person(person)
                })
            })
            .collect();

        (!authors.is_empty()).then_some(authors)
    })
}

/// Extract abstract from a Work
fn extract_abstract(work: &Work) -> Option<Vec<Block>> {
    work.abstract_inverted_index
        .as_ref()
        .and_then(de_invert_abstract)
}

/// Extract publication info from a Work
fn extract_publication_info(work: &Work) -> Option<Box<CreativeWorkVariant>> {
    work.primary_location.as_ref().and_then(|primary_location| {
        create_publication_info(primary_location.source.as_ref(), work.biblio.as_ref())
    })
}

/// Extract page information from a Work
fn extract_page_info(work: &Work) -> (Option<IntegerOrString>, Option<IntegerOrString>) {
    work.biblio
        .as_ref()
        .map(|biblio| {
            (
                biblio.first_page.as_ref().map(IntegerOrString::from),
                biblio.last_page.as_ref().map(IntegerOrString::from),
            )
        })
        .unwrap_or((None, None))
}

/// Extract volume and issue numbers from a Work
fn extract_volume_issue(work: &Work) -> (Option<IntegerOrString>, Option<IntegerOrString>) {
    work.biblio
        .as_ref()
        .map(|biblio| {
            (
                biblio.volume.as_ref().map(IntegerOrString::from),
                biblio.issue.as_ref().map(IntegerOrString::from),
            )
        })
        .unwrap_or((None, None))
}

/// Extract licenses from a Work
fn extract_licenses(work: &Work) -> Option<Vec<CreativeWorkVariantOrString>> {
    work.primary_location.as_ref().and_then(|primary_location| {
        primary_location.license.as_ref().map(|license_str| {
            let normalized_license = normalize_license(license_str);
            vec![CreativeWorkVariantOrString::String(normalized_license)]
        })
    })
}

/// Map OpenAlex work type string to CreativeWorkType enum
fn map_work_type(work_type: Option<&String>) -> Option<CreativeWorkType> {
    work_type.and_then(|wt| match wt.as_str() {
        "article" => Some(CreativeWorkType::Article),
        "book" => Some(CreativeWorkType::Book),
        "book-chapter" => Some(CreativeWorkType::Chapter),
        "dataset" => Some(CreativeWorkType::Dataset),
        "dissertation" => Some(CreativeWorkType::Thesis),
        "report" => Some(CreativeWorkType::Report),
        "webpage" => Some(CreativeWorkType::WebPage),
        "presentation" => Some(CreativeWorkType::Presentation),
        "poster" => Some(CreativeWorkType::Poster),
        "preprint" => Some(CreativeWorkType::Article),
        "review" => Some(CreativeWorkType::Review),
        "editorial" => Some(CreativeWorkType::Article),
        "letter" => Some(CreativeWorkType::Article),
        "erratum" => Some(CreativeWorkType::Article),
        "other" => None,
        _ => None,
    })
}

/// Create a simple Reference for is_part_of field
fn create_reference_part_of(work: &Work) -> Option<Box<Reference>> {
    work.primary_location.as_ref().and_then(|location| {
        location.source.as_ref().map(|source| {
            let title = source
                .display_name
                .as_ref()
                .map(|name| vec![Inline::Text(name.clone().into())]);

            Box::new(Reference {
                work_type: Some(CreativeWorkType::Periodical),
                title,
                ..Default::default()
            })
        })
    })
}

/// Create publication hierarchy from OpenAlex biblio information
fn create_publication_info(
    source: Option<&DehydratedSource>,
    biblio: Option<&Biblio>,
) -> Option<Box<CreativeWorkVariant>> {
    // Get periodical name from source
    let periodical_name = source
        .and_then(|s| s.display_name.as_ref())
        .cloned()
        .unwrap_or_else(|| "Unknown Publication".to_string());

    let periodical = Periodical {
        name: Some(periodical_name),
        ..Default::default()
    };

    if let Some(bib) = biblio {
        if let Some(volume) = &bib.volume {
            let publication_volume = PublicationVolume {
                is_part_of: Some(Box::new(CreativeWorkVariant::Periodical(periodical))),
                volume_number: Some(IntegerOrString::from(volume)),
                ..Default::default()
            };

            if let Some(issue) = &bib.issue {
                let publication_issue = PublicationIssue {
                    is_part_of: Some(Box::new(CreativeWorkVariant::PublicationVolume(
                        publication_volume,
                    ))),
                    issue_number: Some(IntegerOrString::from(issue)),
                    ..Default::default()
                };

                Some(Box::new(CreativeWorkVariant::PublicationIssue(
                    publication_issue,
                )))
            } else {
                Some(Box::new(CreativeWorkVariant::PublicationVolume(
                    publication_volume,
                )))
            }
        } else {
            // No volume, just periodical
            Some(Box::new(CreativeWorkVariant::Periodical(periodical)))
        }
    } else {
        // No biblio, just periodical
        Some(Box::new(CreativeWorkVariant::Periodical(periodical)))
    }
}

/// Trim "Abstract" prefix from abstract text
///
/// Removes variations of "Abstract" from the beginning of abstract text,
/// including "Abstract", "ABSTRACT", "Abstract.", "Abstract:", etc.
fn trim_abstract_prefix(text: &str) -> String {
    let trimmed = text.trim();
    let lowercased = trimmed.to_lowercase();

    // Check for common abstract prefixes (case-insensitive, ordered by specificity)
    let prefixes = [
        "abstract:",
        "abstract.",
        "abstract -",
        "abstract–", // em dash
        "abstract—", // em dash variant
        "abstract",  // This should be last to avoid matching partial words
    ];

    for prefix in &prefixes {
        if lowercased.starts_with(prefix) {
            // Use the original case by slicing from the original string
            let remaining = &trimmed[prefix.len()..];
            return remaining.trim().to_string();
        }
    }

    // Return original text if no prefix found
    trimmed.to_string()
}

/// De-invert an abstract inverted index into readable text
fn de_invert_abstract(inverted_index: &HashMap<String, Vec<i32>>) -> Option<Vec<Block>> {
    if inverted_index.is_empty() {
        return None;
    }

    // Create a vector to hold words at their positions
    let mut words_by_position: Vec<(i32, String)> = Vec::new();

    // Collect all words with their positions
    for (word, positions) in inverted_index {
        for &position in positions {
            words_by_position.push((position, word.clone()));
        }
    }

    // Sort by position
    words_by_position.sort_by_key(|(pos, _)| *pos);

    // Join words into a single string
    let abstract_text = words_by_position
        .into_iter()
        .map(|(_, word)| word)
        .collect::<Vec<_>>()
        .join(" ");

    if abstract_text.trim().is_empty() {
        None
    } else {
        // Trim "Abstract" prefix before creating the paragraph
        let cleaned_text = trim_abstract_prefix(&abstract_text);
        if cleaned_text.is_empty() {
            None
        } else {
            Some(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
                cleaned_text.into(),
            )]))])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_abstract_prefix() {
        // Test various "Abstract" prefixes
        assert_eq!(
            trim_abstract_prefix("Abstract: This is the main content."),
            "This is the main content."
        );

        assert_eq!(
            trim_abstract_prefix("Abstract. This is the main content."),
            "This is the main content."
        );

        assert_eq!(
            trim_abstract_prefix("Abstract This is the main content."),
            "This is the main content."
        );

        assert_eq!(
            trim_abstract_prefix("ABSTRACT: This is the main content."),
            "This is the main content."
        );

        assert_eq!(
            trim_abstract_prefix("abstract - This is the main content."),
            "This is the main content."
        );

        assert_eq!(
            trim_abstract_prefix("Abstract– This is the main content."),
            "This is the main content."
        );

        // Test text without abstract prefix
        assert_eq!(
            trim_abstract_prefix("This research study examines..."),
            "This research study examines..."
        );

        // Test edge cases
        assert_eq!(trim_abstract_prefix("Abstract"), "");
        assert_eq!(trim_abstract_prefix("Abstract:"), "");
        assert_eq!(trim_abstract_prefix("Abstract."), "");
        assert_eq!(trim_abstract_prefix("  Abstract:  "), "");

        // Test that "abstract" in the middle is not trimmed
        assert_eq!(
            trim_abstract_prefix("This abstract contains the word abstract."),
            "This abstract contains the word abstract."
        );
    }

    #[test]
    fn test_de_invert_abstract_with_prefix() {
        // Test that de_invert_abstract properly trims "Abstract" prefix
        let mut inverted_index = HashMap::new();
        inverted_index.insert("Abstract:".to_string(), vec![0]);
        inverted_index.insert("This".to_string(), vec![1]);
        inverted_index.insert("is".to_string(), vec![2]);
        inverted_index.insert("the".to_string(), vec![3]);
        inverted_index.insert("main".to_string(), vec![4]);
        inverted_index.insert("content.".to_string(), vec![5]);

        let result = de_invert_abstract(&inverted_index);
        assert!(result.is_some());

        if let Some(blocks) = result {
            assert_eq!(blocks.len(), 1);
            if let Block::Paragraph(paragraph) = &blocks[0] {
                assert_eq!(paragraph.content.len(), 1);
                if let Inline::Text(text) = &paragraph.content[0] {
                    assert_eq!(text.value, "This is the main content.".into());
                }
            }
        }
    }

    #[test]
    fn test_de_invert_abstract_without_prefix() {
        // Test that de_invert_abstract works normally without prefix
        let mut inverted_index = HashMap::new();
        inverted_index.insert("This".to_string(), vec![0]);
        inverted_index.insert("research".to_string(), vec![1]);
        inverted_index.insert("examines".to_string(), vec![2]);
        inverted_index.insert("methods.".to_string(), vec![3]);

        let result = de_invert_abstract(&inverted_index);
        assert!(result.is_some());

        if let Some(blocks) = result {
            assert_eq!(blocks.len(), 1);
            if let Block::Paragraph(paragraph) = &blocks[0] {
                assert_eq!(paragraph.content.len(), 1);
                if let Inline::Text(text) = &paragraph.content[0] {
                    assert_eq!(text.value, "This research examines methods.".into());
                }
            }
        }
    }
}
