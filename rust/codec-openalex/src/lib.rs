use codec::{
    Codec, DecodeInfo, DecodeOptions,
    common::{
        async_trait::async_trait,
        eyre::{Result, bail, eyre},
        serde::de::DeserializeOwned,
        serde_json,
    },
    schema::Node,
    status::Status,
};

mod author;
mod client;
mod entity;
mod funder;
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
    request, request_ids, search_authors, search_institutions, search_works, list_url,
    work_by_doi,
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

    fn status(&self) -> Status {
        Status::Beta
    }

    async fn from_str(
        &self,
        json: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        Ok((from_str_any(json)?, DecodeInfo::none()))
    }
}

/// Decode a Stencila [`Node`] from an OpenAlex response JSON of known type
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
            from_value_any(first_entity_value)?
        } else {
            bail!("Empty OpenAlex response list")
        }
    } else {
        // Handle single entity response
        from_value_any(&value)?
    };

    Ok(node)
}

/// Decode a Stencila [`Node`] from a [`serde_json::Value`] in an OpenAlex response JSON of unknown type
pub fn from_value_any(value: &serde_json::Value) -> Result<Node> {
    let id = value
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| eyre!("Missing or invalid 'id' field in OpenAlex entity"))?;

    let entity_type = detect_entity_type(id)
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

/// Determine entity type from OpenAlex ID
fn detect_entity_type(id: &str) -> Option<&str> {
    if let Some(id_part) = id.strip_prefix("https://openalex.org/") {
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
}

/// Strip ORCID URL prefix to get just the identifier
pub fn strip_orcid_prefix(orcid: Option<String>) -> Option<String> {
    orcid.and_then(|id| {
        id.strip_prefix("https://orcid.org/")
            .or_else(|| id.strip_prefix("http://orcid.org/"))
            .map(|stripped| stripped.to_string())
            .or(Some(id)) // Return original if no prefix found
    })
}

/// Strip DOI URL prefix to get just the identifier
pub fn strip_doi_prefix(doi: Option<String>) -> Option<String> {
    doi.and_then(|id| {
        id.strip_prefix("https://doi.org/")
            .or_else(|| id.strip_prefix("http://doi.org/"))
            .or_else(|| id.strip_prefix("doi:"))
            .map(|stripped| stripped.to_string())
            .or(Some(id)) // Return original if no prefix found
    })
}

/// Strip ROR URL prefix to get just the identifier
pub fn strip_ror_prefix(ror: Option<String>) -> Option<String> {
    ror.and_then(|id| {
        id.strip_prefix("https://ror.org/")
            .or_else(|| id.strip_prefix("http://ror.org/"))
            .map(|stripped| stripped.to_string())
            .or(Some(id)) // Return original if no prefix found
    })
}
