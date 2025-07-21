use codec::{
    common::serde::Deserialize,
    schema::{
        Article, Block, CreativeWork, CreativeWorkType, Date, IntegerOrString, Organization,
        Periodical, Person, PublicationIssue, PublicationVolume,
    },
};
use std::collections::HashMap;

/// An OpenAlex `Work` object
///
/// See https://docs.openalex.org/api-entities/works/work-object
#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct Work {
    pub id: String,
    pub display_name: Option<String>,
    pub title: Option<String>,
    pub doi: Option<String>,
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
}

/// An OpenAlex `Authorship` object
///
/// See https://docs.openalex.org/api-entities/works/work-object/authorship-object
#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
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
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct DehydratedAuthor {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub orcid: Option<String>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct DehydratedInstitution {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub ror: Option<String>,
    pub country_code: Option<String>,
    pub r#type: Option<String>,
    pub lineage: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct OpenAccess {
    pub is_oa: Option<bool>,
    pub oa_date: Option<String>,
    pub oa_url: Option<String>,
    pub any_repository_has_fulltext: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct Biblio {
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub first_page: Option<String>,
    pub last_page: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct Location {
    pub source: Option<DehydratedSource>,
    pub landing_page_url: Option<String>,
    pub pdf_url: Option<String>,
    pub is_oa: Option<bool>,
    pub version: Option<String>,
    pub license: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
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

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct SustainableDevelopmentGoal {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub score: Option<f64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct Grant {
    pub funder: Option<String>,
    pub funder_display_name: Option<String>,
    pub award_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct CountsByYear {
    pub year: Option<i32>,
    pub cited_by_count: Option<i64>,
}

impl From<Work> for Article {
    fn from(work: Work) -> Self {
        let mut article = Article {
            id: Some(work.id.clone()),
            doi: crate::strip_doi_prefix(work.doi.clone()),
            ..Default::default()
        };

        if let Some(title) = work.display_name.clone().or(work.title.clone()) {
            article.title = Some(vec![codec::schema::Inline::Text(title.into())]);
        }

        // De-invert abstract if present
        if let Some(ref abstract_index) = work.abstract_inverted_index {
            article.r#abstract = de_invert_abstract(abstract_index);
        }

        if let Some(pub_date) = work.publication_date.clone() {
            article.date_published = Some(Date::new(pub_date));
        }

        if let Some(authorships) = &work.authorships {
            let authors: Vec<codec::schema::Author> = authorships
                .iter()
                .filter_map(|authorship| {
                    authorship.author.as_ref().map(|dehydrated_author| {
                        let mut person = Person::default();
                        if let Some(name) = &dehydrated_author.display_name {
                            person.options.name = Some(name.clone());
                        }
                        person.orcid = crate::strip_orcid_prefix(dehydrated_author.orcid.clone());

                        if let Some(institutions) = &authorship.institutions {
                            let orgs: Vec<Organization> = institutions
                                .iter()
                                .map(|inst| Organization {
                                    name: inst.display_name.clone(),
                                    ror: crate::strip_ror_prefix(inst.ror.clone()),
                                    ..Default::default()
                                })
                                .collect();
                            person.affiliations = if orgs.is_empty() { None } else { Some(orgs) };
                        }

                        codec::schema::Author::Person(person)
                    })
                })
                .collect();

            if !authors.is_empty() {
                article.authors = Some(authors);
            }
        }

        // Set page start and end from biblio
        if let Some(biblio) = &work.biblio {
            if let Some(first_page) = &biblio.first_page {
                article.options.page_start = Some(IntegerOrString::String(first_page.clone()));
            }
            if let Some(last_page) = &biblio.last_page {
                article.options.page_end = Some(IntegerOrString::String(last_page.clone()));
            }
        }

        // Create publication info from primary_location source and biblio
        // Don't include page fields in publication hierarchy for articles since they're on the article itself
        if let Some(primary_location) = &work.primary_location {
            if let Some(publication_info) =
                create_publication_info(primary_location.source.as_ref(), work.biblio.as_ref())
            {
                article.options.is_part_of = Some(*publication_info);
            }
        }

        article
    }
}

impl From<Work> for CreativeWork {
    fn from(work: Work) -> Self {
        let mut creative_work = CreativeWork {
            id: Some(work.id),
            doi: crate::strip_doi_prefix(work.doi),
            ..Default::default()
        };

        if let Some(title) = work.display_name.or(work.title) {
            creative_work.options.title = Some(vec![codec::schema::Inline::Text(title.into())]);
        }

        // De-invert abstract if present
        if let Some(ref abstract_index) = work.abstract_inverted_index {
            creative_work.options.r#abstract = de_invert_abstract(abstract_index);
        }

        if let Some(pub_date) = work.publication_date {
            creative_work.options.date_published = Some(Date::new(pub_date));
        }

        if let Some(authorships) = work.authorships {
            let authors: Vec<codec::schema::Author> = authorships
                .into_iter()
                .filter_map(|authorship| {
                    authorship.author.map(|dehydrated_author| {
                        let mut person = Person::default();
                        if let Some(name) = dehydrated_author.display_name {
                            person.options.name = Some(name);
                        }
                        person.orcid = crate::strip_orcid_prefix(dehydrated_author.orcid);

                        if let Some(institutions) = authorship.institutions {
                            let orgs: Vec<Organization> = institutions
                                .into_iter()
                                .map(|inst| Organization {
                                    name: inst.display_name,
                                    ror: crate::strip_ror_prefix(inst.ror),
                                    ..Default::default()
                                })
                                .collect();
                            person.affiliations = if orgs.is_empty() { None } else { Some(orgs) };
                        }

                        codec::schema::Author::Person(person)
                    })
                })
                .collect();

            if !authors.is_empty() {
                creative_work.options.authors = Some(authors);
            }
        }

        // Create publication info from primary_location source and biblio
        // Include page fields in publication hierarchy for non-article creative works
        if let Some(primary_location) = &work.primary_location {
            if let Some(publication_info) =
                create_publication_info(primary_location.source.as_ref(), work.biblio.as_ref())
            {
                creative_work.options.is_part_of = Some(*publication_info);
            }
        }

        creative_work
    }
}

/// Create publication hierarchy from OpenAlex biblio information
fn create_publication_info(
    source: Option<&DehydratedSource>,
    biblio: Option<&Biblio>,
) -> Option<Box<CreativeWorkType>> {
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
        if let Some(volume_str) = &bib.volume {
            let publication_volume = PublicationVolume {
                is_part_of: Some(Box::new(CreativeWorkType::Periodical(periodical))),
                volume_number: Some(IntegerOrString::String(volume_str.clone())),
                ..Default::default()
            };

            if let Some(issue_str) = &bib.issue {
                let publication_issue = PublicationIssue {
                    is_part_of: Some(Box::new(CreativeWorkType::PublicationVolume(
                        publication_volume,
                    ))),
                    issue_number: Some(IntegerOrString::String(issue_str.clone())),
                    ..Default::default()
                };

                Some(Box::new(CreativeWorkType::PublicationIssue(
                    publication_issue,
                )))
            } else {
                Some(Box::new(CreativeWorkType::PublicationVolume(
                    publication_volume,
                )))
            }
        } else {
            // No volume, just periodical
            Some(Box::new(CreativeWorkType::Periodical(periodical)))
        }
    } else {
        // No biblio, just periodical
        Some(Box::new(CreativeWorkType::Periodical(periodical)))
    }
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
        Some(vec![Block::Paragraph(codec::schema::Paragraph::new(vec![
            codec::schema::Inline::Text(abstract_text.into()),
        ]))])
    }
}
