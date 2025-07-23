use codec::{
    common::{eyre::Result, serde::Deserialize},
    schema::{Organization, Person},
};

/// An OpenAlex `Author` object
///
/// See https://docs.openalex.org/api-entities/authors/author-object
#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct Author {
    pub id: String,
    pub orcid: Option<String>,
    pub display_name: Option<String>,
    pub display_name_alternatives: Option<Vec<String>>,
    pub works_count: Option<i64>,
    pub cited_by_count: Option<i64>,
    pub summary_stats: Option<SummaryStats>,
    pub ids: Option<ExternalIds>,
    pub affiliations: Option<Vec<Affiliation>>,
    pub last_known_institutions: Option<Vec<DehydratedInstitution>>,
    pub works_api_url: Option<String>,
    pub updated_date: Option<String>,
    pub created_date: Option<String>,
    pub counts_by_year: Option<Vec<CountsByYear>>,
    pub x_concepts: Option<Vec<Concept>>,
}

impl Author {
    /// Get the ORCID of an author, or generate a pseudo ORCID
    pub fn orcid(&self, prefix: char) -> Result<String> {
        crate::utils::get_or_generate_orcid(&self.orcid, &self.id, prefix)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct SummaryStats {
    #[serde(rename = "2yr_mean_citedness")]
    pub two_yr_mean_citedness: Option<f64>,
    pub h_index: Option<i32>,
    pub i10_index: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct ExternalIds {
    pub openalex: Option<String>,
    pub orcid: Option<String>,
    pub scopus: Option<String>,
    pub twitter: Option<String>,
    pub wikipedia: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct Affiliation {
    pub institution: Option<DehydratedInstitution>,
    pub years: Option<Vec<i32>>,
}

#[derive(Deserialize)]
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
pub struct CountsByYear {
    pub year: Option<i32>,
    pub works_count: Option<i64>,
    pub cited_by_count: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", crate = "codec::common::serde")]
pub struct Concept {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub score: Option<f64>,
}

impl From<Author> for Person {
    fn from(author: Author) -> Self {
        let mut person = Person {
            id: Some(author.id),
            orcid: crate::strip_orcid_prefix(author.orcid),
            ..Default::default()
        };
        person.options.name = author.display_name;

        if let Some(affiliations) = author.affiliations {
            let organizations: Vec<Organization> = affiliations
                .into_iter()
                .filter_map(|affiliation| {
                    affiliation.institution.map(|inst| Organization {
                        id: inst.id,
                        name: inst.display_name,
                        ror: crate::strip_ror_prefix(inst.ror),
                        ..Default::default()
                    })
                })
                .collect();

            if !organizations.is_empty() {
                person.affiliations = Some(organizations);
            }
        } else if let Some(last_known) = author.last_known_institutions {
            let organizations: Vec<Organization> = last_known
                .into_iter()
                .map(|inst| Organization {
                    id: inst.id,
                    name: inst.display_name,
                    ror: crate::strip_ror_prefix(inst.ror),
                    ..Default::default()
                })
                .collect();

            if !organizations.is_empty() {
                person.affiliations = Some(organizations);
            }
        }

        person
    }
}
