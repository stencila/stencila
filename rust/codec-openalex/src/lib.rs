use serde::de::DeserializeOwned;

use stencila_codec::{
    Codec, DecodeInfo, DecodeOptions, async_trait,
    eyre::{Result, bail, eyre},
    stencila_schema::{Node, Reference},
};
use stencila_codec_text::to_text;

mod author;
mod client;
mod entity;
mod funder;
mod ids;
mod institution;
mod license;
mod publisher;
mod responses;
mod source;
mod utils;
mod work;

use author::Author;
use funder::Funder;
use institution::Institution;
use publisher::Publisher;
use source::Source;
use work::Work;

// Re-export client functions
pub use client::{
    list_url, request_ids, request_list, search_authors, search_institutions,
    search_works_title_year, work_by_doi,
};

// Re-export types that might be needed by consumers
pub use author::Author as OpenAlexAuthor;
pub use institution::Institution as OpenAlexInstitution;
pub use responses::{
    AuthorsResponse, FundersResponse, InstitutionsResponse, PublishersResponse, SelectResponse,
    SourcesResponse, WorksResponse,
};
pub use source::Source as OpenAlexSource;
pub use work::Work as OpenAlexWork;

use client::fetch_work_references;

/// A codec for decoding OpenAlex API response JSON to Stencila Schema nodes
///
/// Not exposed as a standalone codec but used by sibling creates that
/// make use of the OpenAlex API.
///
/// See https://docs.openalex.org/ for details.
pub struct OpenAlexCodec;

#[async_trait]
impl Codec for OpenAlexCodec {
    fn name(&self) -> &str {
        "openalex"
    }

    async fn from_str(
        &self,
        json: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        Ok((Self::from_str_any(json)?, DecodeInfo::none()))
    }
}

impl OpenAlexCodec {
    #[tracing::instrument(skip(reference))]
    pub async fn from_reference(reference: &Reference) -> Result<Node> {
        // If the reference has a DOI, then try to use that
        if let Some(doi) = &reference.doi {
            tracing::debug!("Getting work by DOI");

            // Get work by DOI, returning early if successful
            if let Some(mut work) = work_by_doi(doi).await? {
                fetch_work_references(&mut work).await?;
                return Ok(work.into());
            };
        }

        // If the reference does not have a DOI, or the above failed, search by title and year
        let mut works = if let Some(title) = &reference.title {
            let year = reference
                .date
                .as_ref()
                .and_then(|date| date.year())
                .map(|year| year as i32);
            search_works_title_year(&to_text(title), year).await?
        } else if let Some(text) = &reference.options.text {
            search_works_title_year(text, None).await?
        } else {
            bail!("Reference has no title or text to search using");
        };

        if works.is_empty() {
            bail!("No works matched reference");
        };

        let mut work = works.swap_remove(0);
        fetch_work_references(&mut work).await?;

        Ok(work.into())
    }

    /// Decode a Stencila [`Node`] from an OpenAlex response JSON of known type
    #[allow(clippy::should_implement_trait)]
    pub fn from_str<T>(json: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        Ok(serde_json::from_str(json)?)
    }

    /// Decode a Stencila [`Node`] from an OpenAlex response JSON of unknown type
    pub fn from_str_any(json: &str) -> Result<Node> {
        // Parse as generic JSON first
        let value: serde_json::Value = serde_json::from_str(json)?;

        let node = if let Some(results) = value.get("results") {
            // Handle list response - take the first entity
            if let Some(first_entity_value) = results.as_array().and_then(|arr| arr.first()) {
                Self::from_value_any(first_entity_value)?
            } else {
                bail!("Empty OpenAlex response list")
            }
        } else {
            // Handle single entity response
            Self::from_value_any(&value)?
        };

        Ok(node)
    }

    /// Decode a Stencila [`Node`] from a [`serde_json::Value`] in an OpenAlex response JSON of unknown type
    pub fn from_value_any(value: &serde_json::Value) -> Result<Node> {
        let id = value
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| eyre!("Missing or invalid 'id' field in OpenAlex entity"))?;

        let entity_type = if let Some(id_part) = id.strip_prefix("https://openalex.org/") {
            match id_part.chars().next() {
                Some('A') => Some("author"),
                Some('W') => Some("work"),
                Some('S') => Some("source"),
                Some('I') => Some("institution"),
                Some('F') => Some("funder"),
                Some('P') => Some("publisher"),
                _ => None,
            }
        } else {
            None
        }
        .ok_or_else(|| eyre!("Unable to determine entity type from ID: {id}"))?;

        match entity_type {
            "author" => {
                let author: Author = serde_json::from_value(value.clone())?;
                Ok(Node::Person(author.into()))
            }
            "work" => {
                let work: Work = serde_json::from_value(value.clone())?;
                if work.r#type.as_deref() == Some("article") {
                    Ok(Node::Article(work.into()))
                } else {
                    Ok(Node::CreativeWork(work.into()))
                }
            }
            "source" => {
                let source: Source = serde_json::from_value(value.clone())?;
                Ok(Node::Periodical(source.into()))
            }
            "institution" => {
                let institution: Institution = serde_json::from_value(value.clone())?;
                Ok(Node::Organization(institution.into()))
            }
            "funder" => {
                let funder: Funder = serde_json::from_value(value.clone())?;
                Ok(Node::Organization(funder.into()))
            }
            "publisher" => {
                let publisher: Publisher = serde_json::from_value(value.clone())?;
                Ok(Node::Organization(publisher.into()))
            }
            _ => bail!("Unsupported entity type: {entity_type}"),
        }
    }
}
