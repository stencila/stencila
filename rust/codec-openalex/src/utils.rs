use indexmap::IndexMap;
use stencila_codec::{
    eyre::Result,
    stencila_schema::{Primitive, PropertyValue, PropertyValueOrString},
};

/// Strip the OpenAlex ID prefix
pub fn strip_openalex_prefix(id: &str) -> String {
    id.trim_start_matches("https://openalex.org/").into()
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

/// Strip ORCID URL prefix to get just the identifier
pub fn strip_orcid_prefix(orcid: Option<String>) -> Option<String> {
    orcid.and_then(|id| {
        id.strip_prefix("https://orcid.org/")
            .or_else(|| id.strip_prefix("http://orcid.org/"))
            .map(|stripped| stripped.to_string())
            .or(Some(id)) // Return original if no prefix found
    })
}

/// Get ORCID from an optional field, or generate a pseudo-ORCID from OpenAlex ID
pub fn get_or_generate_orcid(
    orcid: &Option<String>,
    openalex_id: &str,
    prefix: char,
) -> Result<String> {
    if let Some(orcid) = orcid {
        Ok(orcid.trim_start_matches("https://orcid.org/").into())
    } else {
        generate_pseudo_orcid(openalex_id, prefix)
    }
}

/// Generate a pseudo-ORCID from an OpenAlex ID
pub fn generate_pseudo_orcid(openalex_id: &str, prefix: char) -> Result<String> {
    let int: u64 = openalex_id
        .trim_start_matches("https://openalex.org/")
        .trim_start_matches("A")
        .parse()?;
    let digits = format!("{:015}", int % 1_000_000_000_000_000);
    Ok(format!(
        "{prefix}{}-{}-{}-{}",
        &digits[0..3],
        &digits[3..7],
        &digits[7..11],
        &digits[11..15],
    ))
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

/// Get ROR from an optional field, or generate a pseudo-ROR from OpenAlex ID
pub fn get_or_generate_ror(ror: &Option<String>, openalex_id: &str, prefix: char) -> String {
    if let Some(ror) = ror {
        ror.trim_start_matches("https://ror.org/").into()
    } else {
        generate_pseudo_ror(openalex_id, prefix)
    }
}
/// Generate a pseudo-ROR from an OpenAlex ID  
pub fn generate_pseudo_ror(openalex_id: &str, prefix: char) -> String {
    let digits = openalex_id
        .trim_start_matches("https://openalex.org/")
        .trim_start_matches('I');
    format!("{prefix}{digits}")
}

/// Convert OpenAlex ids to Stencila identifiers
pub fn convert_ids_to_identifiers(
    ids: &IndexMap<String, String>,
) -> Option<Vec<PropertyValueOrString>> {
    if ids.is_empty() {
        return None;
    }

    let identifiers: Vec<PropertyValueOrString> = ids
        .iter()
        .map(|(property_id, value)| {
            // If the value is a URL, use it directly as a string identifier
            if value.starts_with("http://") || value.starts_with("https://") {
                PropertyValueOrString::String(value.clone())
            } else {
                // Otherwise create a PropertyValue with property_id and value
                PropertyValueOrString::PropertyValue(PropertyValue {
                    property_id: Some(property_id.clone()),
                    value: Primitive::String(value.clone()),
                    ..Default::default()
                })
            }
        })
        .collect();

    if identifiers.is_empty() {
        None
    } else {
        Some(identifiers)
    }
}
