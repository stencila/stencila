use std::{path::PathBuf, str::FromStr};

use codec::{
    format::Format,
    schema::{
        Article, ArticleOptions, Author, Block, CreativeWorkTypeOrString, Datatable,
        DatatableOptions, Date, Grant, GrantOptions, GrantOrMonetaryGrant, Inline, Node,
        Organization, Paragraph, Person, PersonOptions, PropertyValueOrString, Reference,
        SoftwareSourceCode, SoftwareSourceCodeOptions, StringOrNumber, Text,
    },
};
use codec_text::to_text;

use crate::responses::Record;

/// Parse HTML content into Stencila blocks, or fallback to plain text paragraph
fn parse_html_or_text(content: &str) -> Vec<Block> {
    // Check if content contains HTML tags
    if content.contains('<') && content.contains('>') {
        // Try to parse as HTML
        match codec_html::decode(content, None) {
            Ok((Node::Article(article), _)) => {
                if !article.content.is_empty() {
                    return article.content;
                }
            }
            _ => {
                // HTML parsing failed, fallback to text
            }
        }
    }

    // Fallback: treat as plain text
    vec![Block::Paragraph(Paragraph {
        content: vec![Inline::Text(Text::from(content.to_string()))],
        ..Default::default()
    })]
}

/// Strip HTML tags from text and return plain text
fn strip_html_tags(content: &str) -> String {
    // Check if content contains HTML tags
    if content.contains('<') && content.contains('>') {
        // Try to parse as HTML and extract text content
        match codec_html::decode(content, None) {
            Ok((Node::Article(article), _)) => {
                // Extract all text from the blocks
                to_text(&article)
            }
            _ => {
                // HTML parsing failed, return original content
                content.to_string()
            }
        }
    } else {
        // No HTML tags, return as-is
        content.to_string()
    }
}

/// Convert Zenodo related identifiers to Stencila references
fn related_identifiers_to_references(record: &Record) -> Vec<Reference> {
    record
        .metadata
        .related_identifiers
        .iter()
        .map(|related| {
            // Use the identifier as the title (could be a URL or DOI)
            let title = Some(vec![Inline::Text(Text::from(related.identifier.clone()))]);

            // If it's a DOI, extract it
            let doi = if related.scheme.as_deref() == Some("doi") {
                Some(related.identifier.clone())
            } else {
                None
            };

            Reference {
                title,
                doi,
                ..Default::default()
            }
        })
        .collect()
}

/// Convert Zenodo bibliographic references to Stencila references
fn bibliographic_references_to_references(record: &Record) -> Vec<Reference> {
    record
        .metadata
        .references
        .iter()
        .map(|ref_string| {
            // Use the reference string as the title
            Reference {
                title: Some(vec![Inline::Text(Text::from(ref_string.clone()))]),
                ..Default::default()
            }
        })
        .collect()
}

/// Combine all types of references (related identifiers + bibliographic references)
fn all_references(record: &Record) -> Option<Vec<Reference>> {
    let mut all_refs = Vec::new();

    // Add related identifiers
    all_refs.extend(related_identifiers_to_references(record));

    // Add bibliographic references
    all_refs.extend(bibliographic_references_to_references(record));

    if all_refs.is_empty() {
        None
    } else {
        Some(all_refs)
    }
}

/// Convert Zenodo contributor to Stencila Author (reuses same logic as creators)
fn contributor_to_author(contributor: &crate::responses::Contributor) -> Author {
    // Use Person::from_str to parse the name, then merge with other props
    let (parsed, name) = (
        Person::from_str(&contributor.name).ok(),
        Some(contributor.name.clone()),
    );

    // Create affiliations if present
    let affiliations = contributor.affiliation.as_ref().map(|affiliation| {
        vec![Organization {
            name: Some(affiliation.clone()),
            ..Default::default()
        }]
    });

    Author::Person(Person {
        orcid: contributor.orcid.clone(),
        affiliations,
        options: Box::new(PersonOptions {
            name,
            ..Default::default()
        }),
        // Parsed names may include given names, family name, honorifics etc
        ..parsed.unwrap_or_default()
    })
}

/// Convert Zenodo grant to Stencila GrantOrMonetaryGrant
fn grant_to_grant_or_monetary_grant(grant: &crate::responses::Grant) -> GrantOrMonetaryGrant {
    GrantOrMonetaryGrant::Grant(Grant {
        options: Box::new(GrantOptions {
            name: grant.title.clone().or_else(|| {
                // Use funder name + code as fallback name
                grant.funder.as_ref().and_then(|funder| {
                    let funder_name = funder.name.as_ref()?;
                    if let Some(code) = &grant.code {
                        Some(format!("{funder_name} ({code})"))
                    } else {
                        Some(funder_name.clone())
                    }
                })
            }),
            description: grant.title.clone(), // Use title as description too
            identifiers: grant
                .code
                .as_ref()
                .map(|code| vec![PropertyValueOrString::String(code.clone())]),
            ..Default::default()
        }),
        ..Default::default()
    })
}

impl From<Record> for Node {
    fn from(record: Record) -> Self {
        // Convert based on resource type
        match record.metadata.resource_type.type_.as_str() {
            "publication" | "poster" | "presentation" | "thesis" | "report" => {
                Node::Article(record.into())
            }
            "dataset" => Node::Datatable(record.into()),
            "software" => Node::SoftwareSourceCode(record.into()),
            _ => {
                // Default to article for other types
                Node::Article(record.into())
            }
        }
    }
}

impl From<Record> for Article {
    fn from(record: Record) -> Self {
        let metadata = &record.metadata;

        // Convert creators to Author nodes
        let authors = metadata
            .creators
            .iter()
            .map(|creator| {
                // Use Person::from_str to parse the name, then merge with other props
                let (parsed, name) = (
                    Person::from_str(&creator.name).ok(),
                    Some(creator.name.clone()),
                );

                // Create affiliations if present
                let affiliations = creator.affiliation.as_ref().map(|affiliation| {
                    vec![Organization {
                        name: Some(affiliation.clone()),
                        ..Default::default()
                    }]
                });

                Author::Person(Person {
                    orcid: creator.orcid.clone(),
                    affiliations,
                    options: Box::new(PersonOptions {
                        name,
                        ..Default::default()
                    }),
                    // Parsed names may include given names, family name, honorifics etc
                    ..parsed.unwrap_or_default()
                })
            })
            .collect();

        Article {
            title: Some(vec![Inline::Text(Text::from(metadata.title.clone()))]),
            authors: Some(authors),
            r#abstract: metadata
                .description
                .as_ref()
                .map(|desc| parse_html_or_text(desc)),
            keywords: {
                let mut all_keywords = metadata.keywords.clone();
                // Add communities as keywords with a prefix
                for community in &metadata.communities {
                    all_keywords.push(format!("community:{}", community.id));
                }
                if !all_keywords.is_empty() {
                    Some(all_keywords)
                } else {
                    None
                }
            },
            date_published: Some(Date {
                value: metadata.publication_date.clone(),
                ..Default::default()
            }),
            doi: metadata.doi.clone(),
            references: all_references(&record),
            options: Box::new(ArticleOptions {
                licenses: metadata
                    .license
                    .as_ref()
                    .map(|license| vec![CreativeWorkTypeOrString::String(license.id.clone())]),
                contributors: if !metadata.contributors.is_empty() {
                    Some(
                        metadata
                            .contributors
                            .iter()
                            .map(contributor_to_author)
                            .collect(),
                    )
                } else {
                    None
                },
                funded_by: if !metadata.grants.is_empty() {
                    Some(
                        metadata
                            .grants
                            .iter()
                            .map(grant_to_grant_or_monetary_grant)
                            .collect(),
                    )
                } else {
                    None
                },
                ..Default::default()
            }),
            content: Vec::new(),
            ..Default::default()
        }
    }
}

impl From<Record> for Datatable {
    fn from(record: Record) -> Self {
        let metadata = &record.metadata;

        // Get the first CSV/TSV file if available
        let data_file = record.files.iter().find(|f| {
            let path = PathBuf::from(&f.key);
            matches!(
                Format::from_path(&path),
                Format::Csv | Format::Tsv | Format::Xlsx | Format::Xls | Format::Ods
            )
        });

        // Get DOI URL
        let doi_url = metadata
            .doi
            .as_ref()
            .map(|doi| format!("https://doi.org/{doi}"));

        Datatable {
            doi: metadata.doi.clone(),
            options: Box::new(DatatableOptions {
                name: Some(metadata.title.clone()),
                r#abstract: metadata
                    .description
                    .as_ref()
                    .map(|desc| parse_html_or_text(desc)),
                url: data_file.map(|f| f.links.self_.clone()).or(doi_url),
                version: metadata
                    .version
                    .as_ref()
                    .map(|v| StringOrNumber::String(v.clone())),
                licenses: metadata
                    .license
                    .as_ref()
                    .map(|license| vec![CreativeWorkTypeOrString::String(license.id.clone())]),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

impl From<Record> for SoftwareSourceCode {
    fn from(record: Record) -> Self {
        let metadata = &record.metadata;

        // Convert creators to Author nodes (same logic as Article)
        let authors = if !metadata.creators.is_empty() {
            Some(
                metadata
                    .creators
                    .iter()
                    .map(|creator| {
                        // Use Person::from_str to parse the name, then merge with other props
                        let (parsed, name) = (
                            Person::from_str(&creator.name).ok(),
                            Some(creator.name.clone()),
                        );

                        // Create affiliations if present
                        let affiliations = creator.affiliation.as_ref().map(|affiliation| {
                            vec![Organization {
                                name: Some(affiliation.clone()),
                                ..Default::default()
                            }]
                        });

                        Author::Person(Person {
                            orcid: creator.orcid.clone(),
                            affiliations,
                            options: Box::new(PersonOptions {
                                name,
                                ..Default::default()
                            }),
                            // Parsed names may include given names, family name, honorifics etc
                            ..parsed.unwrap_or_default()
                        })
                    })
                    .collect(),
            )
        } else {
            None
        };

        // Get the first source code file if available
        let code_file = record.files.iter().find(|f| {
            let path = PathBuf::from(&f.key);
            let format = Format::from_path(&path);
            // Check for common programming language files
            matches!(format, Format::Python | Format::JavaScript)
                || f.key.ends_with(".zip")
                || f.key.ends_with(".tar.gz")
        });

        // Get DOI URL
        let doi_url = metadata
            .doi
            .as_ref()
            .map(|doi| format!("https://doi.org/{doi}"));

        // Look for GitHub repository URL in related identifiers
        let github_url = record
            .metadata
            .related_identifiers
            .iter()
            .find(|related| {
                related.scheme.as_deref() == Some("url")
                    && related.identifier.contains("github.com")
            })
            .map(|related| related.identifier.clone());

        // Extract programming language from files
        let programming_language = record.files.iter().find_map(|f| {
            let path = PathBuf::from(&f.key);
            let ext = path.extension()?.to_str()?;
            match ext {
                "py" => Some("Python"),
                "rs" => Some("Rust"),
                "js" | "mjs" => Some("JavaScript"),
                "ts" | "tsx" => Some("TypeScript"),
                "java" => Some("Java"),
                "cpp" | "cxx" | "cc" => Some("C++"),
                "c" => Some("C"),
                "go" => Some("Go"),
                "r" | "R" => Some("R"),
                "jl" => Some("Julia"),
                "m" => Some("MATLAB"),
                _ => None,
            }
            .map(|s| s.to_string())
        });

        SoftwareSourceCode {
            name: metadata.title.clone(),
            programming_language: programming_language.unwrap_or_default(),
            doi: metadata.doi.clone(),
            repository: github_url.clone(),
            version: metadata
                .version
                .as_ref()
                .map(|v| StringOrNumber::String(v.clone())),
            options: Box::new(SoftwareSourceCodeOptions {
                authors,
                description: metadata
                    .description
                    .as_ref()
                    .map(|desc| strip_html_tags(desc)),
                url: github_url
                    .or_else(|| code_file.map(|f| f.links.self_.clone()))
                    .or(doi_url),
                keywords: if !metadata.keywords.is_empty() {
                    Some(metadata.keywords.clone())
                } else {
                    None
                },
                licenses: metadata
                    .license
                    .as_ref()
                    .map(|license| vec![CreativeWorkTypeOrString::String(license.id.clone())]),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
