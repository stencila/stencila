use indexmap::IndexMap;
use stencila_codec::{
    eyre::Result,
    stencila_schema::{Primitive, PropertyValue, PropertyValueOrString},
};

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

/// Generate a pseudo-ROR from an OpenAlex ID  
pub fn generate_pseudo_ror(openalex_id: &str, prefix: char) -> String {
    let digits = openalex_id
        .trim_start_matches("https://openalex.org/")
        .trim_start_matches('I');
    format!("{prefix}{digits}")
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

/// Get ROR from an optional field, or generate a pseudo-ROR from OpenAlex ID
pub fn get_or_generate_ror(ror: &Option<String>, openalex_id: &str, prefix: char) -> String {
    if let Some(ror) = ror {
        ror.trim_start_matches("https://ror.org/").into()
    } else {
        generate_pseudo_ror(openalex_id, prefix)
    }
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
