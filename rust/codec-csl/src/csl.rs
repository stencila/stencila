use std::str::FromStr;

use codec::{
    common::{
        eyre::Result,
        indexmap::IndexMap,
        serde::{Deserialize, Serialize},
        serde_json::Value,
        serde_with::skip_serializing_none,
    },
    schema::{
        Article, Author, CreativeWorkType, Date, IntegerOrString, Organization, Periodical, Person,
        PersonOrOrganization, PublicationIssue, PublicationVolume,
        shortcuts::{p, t},
    },
};

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", crate = "codec::common::serde")]
pub struct CslItem {
    pub id: Option<String>,

    #[serde(rename = "type")]
    pub item_type: String,

    // Common fields
    pub title: Option<String>,
    pub author: Option<Vec<CslName>>,
    pub issued: Option<CslDate>,

    // Publication fields
    pub container_title: Option<Value>,
    pub volume: Option<StringOrNumber>,
    pub issue: Option<StringOrNumber>,
    pub page: Option<String>,
    pub doi: Option<String>,
    pub url: Option<String>,
    pub issn: Option<String>,
    pub isbn: Option<String>,

    // Additional fields
    pub publisher: Option<String>,
    pub publisher_place: Option<String>,
    #[serde(rename = "abstract")]
    pub abstract_text: Option<String>,
    pub language: Option<String>,

    // Catch-all for other fields
    // Uses `IndexMap` so that order is deterministic
    #[serde(flatten)]
    pub other: IndexMap<String, Value>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", crate = "codec::common::serde")]
pub struct CslName {
    pub family: Option<String>,
    pub given: Option<String>,
    pub literal: Option<String>,
    pub dropping_particle: Option<String>,
    pub non_dropping_particle: Option<String>,
    pub suffix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, crate = "codec::common::serde")]
pub enum CslDate {
    DateParts {
        #[serde(rename = "date-parts")]
        date_parts: Vec<Vec<i32>>,
    },
    Literal {
        literal: String,
    },
    Raw {
        raw: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, crate = "codec::common::serde")]
pub enum StringOrNumber {
    String(String),
    Number(i32),
}

impl CslItem {
    /// Convert CSL-JSON item to Stencila [`Article`] node
    pub fn to_article(&self) -> Result<Article> {
        let title = self.title.as_ref().map(|title_str| vec![t(title_str)]);

        let authors = self
            .author
            .as_ref()
            .map(|authors| convert_csl_authors(authors))
            .unwrap_or_default();

        let authors = (!authors.is_empty()).then_some(authors);

        let date_published = self.issued.as_ref().and_then(convert_csl_date);

        let r#abstract = self
            .abstract_text
            .as_ref()
            .map(|text| vec![p(vec![t(text)])]);

        let doi = self.doi.clone();
        let url = self.url.clone();
        let publisher = self.publisher.clone();

        let is_part_of = self.container_title.as_ref().and_then(|container| {
            create_publication_info(
                container,
                self.volume.as_ref(),
                self.issue.as_ref(),
                self.page.as_deref(),
            )
        });

        let mut article = Article {
            title,
            authors,
            r#abstract,
            doi,
            date_published,
            ..Default::default()
        };

        if let Some(url_value) = url {
            article.options.url = Some(url_value);
        }
        if let Some(is_part_of_value) = is_part_of {
            article.options.is_part_of = Some(*is_part_of_value);
        }
        if let Some(publisher_name) = publisher {
            article.options.publisher = Some(PersonOrOrganization::Organization(Organization {
                name: Some(publisher_name),
                ..Default::default()
            }));
        }

        Ok(article)
    }
}

/// Convert CSL authors to Stencila authors
fn convert_csl_authors(csl_authors: &[CslName]) -> Vec<Author> {
    csl_authors
        .iter()
        .filter_map(|csl_author| {
            if let Some(literal) = &csl_author.literal {
                // Try to parse the literal name
                match Person::from_str(literal) {
                    Ok(person) => Some(Author::Person(person)),
                    Err(_) => {
                        // Fall back to literal name as given name
                        Some(Author::Person(Person {
                            given_names: Some(vec![literal.clone()]),
                            ..Default::default()
                        }))
                    }
                }
            } else {
                // Build from parts
                let mut person = Person::default();
                if let Some(given) = &csl_author.given {
                    person.given_names = Some(vec![given.clone()]);
                }
                if let Some(family) = &csl_author.family {
                    person.family_names = Some(vec![family.clone()]);
                }

                // Only create a person if we have at least some name information
                if person.given_names.is_some() || person.family_names.is_some() {
                    Some(Author::Person(person))
                } else {
                    None
                }
            }
        })
        .collect()
}

/// Convert CSL date to Stencila Date
fn convert_csl_date(csl_date: &CslDate) -> Option<Date> {
    match csl_date {
        CslDate::DateParts { date_parts } => {
            if let Some(parts) = date_parts.first() {
                let year = parts.first().map(|y| y.to_string()).unwrap_or_default();
                let month = parts.get(1).map(|m| format!("-{m:02}")).unwrap_or_default();
                let day = parts.get(2).map(|d| format!("-{d:02}")).unwrap_or_default();
                Some(Date::new(format!("{year}{month}{day}")))
            } else {
                None
            }
        }
        CslDate::Literal { literal } => Some(Date::new(literal.to_string())),
        CslDate::Raw { raw } => Some(Date::new(raw.to_string())),
    }
}

/// Create publication hierarchy from CSL metadata
fn create_publication_info(
    container_title: &Value,
    volume: Option<&StringOrNumber>,
    issue: Option<&StringOrNumber>,
    page: Option<&str>,
) -> Option<Box<CreativeWorkType>> {
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
            StringOrNumber::String(s) => s.clone(),
            StringOrNumber::Number(n) => n.to_string(),
        };

        let mut publication_volume = PublicationVolume {
            is_part_of: Some(Box::new(CreativeWorkType::Periodical(periodical))),
            volume_number: Some(IntegerOrString::String(volume_str)),
            ..Default::default()
        };

        if let Some(iss) = issue {
            let issue_str = match iss {
                StringOrNumber::String(s) => s.clone(),
                StringOrNumber::Number(n) => n.to_string(),
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

            Some(Box::new(CreativeWorkType::PublicationIssue(
                publication_issue,
            )))
        } else {
            if let Some(page_range) = page {
                publication_volume.options.page_start = extract_page_start(page_range);
                publication_volume.options.page_end = extract_page_end(page_range);
            }
            Some(Box::new(CreativeWorkType::PublicationVolume(
                publication_volume,
            )))
        }
    } else {
        Some(Box::new(CreativeWorkType::Periodical(periodical)))
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
