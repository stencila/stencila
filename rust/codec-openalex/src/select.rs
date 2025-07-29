use codec::common::serde_json;

/// Response for select API calls with partial fields
///
/// See https://docs.openalex.org/how-to-use-the-api/get-lists-of-entities/select-fields
pub type Select = serde_json::Map<String, serde_json::Value>;
