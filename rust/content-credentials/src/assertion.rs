//! Strongly-typed payload for the `org.stencila.provenance` C2PA assertion.
//!
//! Deliberately minimal in this MVP. Unknown fields received on the wire are
//! preserved via [`ProvenanceAssertion::extra`] so readers built today keep
//! working when the payload schema grows.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::schema::PROVENANCE_SCHEMA_V1;

/// The Stencila provenance assertion payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProvenanceAssertion {
    /// Schema URL identifying the payload version.
    pub schema: String,

    /// Software that produced the asset.
    pub producer: Producer,

    /// The asset to which this assertion is bound.
    pub asset: AssetRecord,

    /// Forward-compatibility slot for future fields.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Producer {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetRecord {
    pub media_type: String,
    #[serde(alias = "digest")]
    pub source_digest: String,
}

impl ProvenanceAssertion {
    /// Construct a v1 assertion for an asset of the given media type and source digest.
    pub fn new_v1(media_type: impl Into<String>, source_digest: impl Into<String>) -> Self {
        Self {
            schema: PROVENANCE_SCHEMA_V1.to_string(),
            producer: Producer {
                name: "Stencila".to_string(),
                version: stencila_version::STENCILA_VERSION.to_string(),
            },
            asset: AssetRecord {
                media_type: media_type.into(),
                source_digest: source_digest.into(),
            },
            extra: Map::new(),
        }
    }

    /// Whether this payload's schema URL is one this build understands.
    #[must_use]
    pub fn is_known_schema(&self) -> bool {
        self.schema == PROVENANCE_SCHEMA_V1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Ensures the minimal v1 provenance assertion round-trips without changing known fields.
    #[test]
    fn round_trip_minimal() {
        let original = ProvenanceAssertion::new_v1("image/png", "sha256:abc");
        let json = serde_json::to_string(&original).expect("serialize");
        let parsed: ProvenanceAssertion = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.schema, PROVENANCE_SCHEMA_V1);
        assert_eq!(parsed.producer.name, "Stencila");
        assert_eq!(parsed.asset.media_type, "image/png");
        assert_eq!(parsed.asset.source_digest, "sha256:abc");
        assert!(parsed.is_known_schema());
        assert!(parsed.extra.is_empty());
    }

    /// Ensures future assertion fields survive deserialization and serialization.
    #[test]
    fn unknown_fields_preserved() {
        // A future payload includes fields this build does not know about.
        let raw = json!({
            "schema": PROVENANCE_SCHEMA_V1,
            "producer": { "name": "Stencila", "version": "9.9.9" },
            "asset": { "mediaType": "image/png", "sourceDigest": "sha256:abc" },
            "newField": "future",
            "nested": { "more": [1, 2, 3] }
        });

        let parsed: ProvenanceAssertion = serde_json::from_value(raw.clone()).expect("deserialize");
        assert_eq!(parsed.extra.get("newField"), Some(&json!("future")));
        assert!(parsed.extra.contains_key("nested"));

        // Round-trip preserves unknown fields.
        let again = serde_json::to_value(&parsed).expect("serialize");
        assert_eq!(again, raw);
    }

    /// Ensures the previous `digest` field name is still accepted for v1 payloads.
    #[test]
    fn legacy_digest_field_is_accepted() {
        let raw = json!({
            "schema": PROVENANCE_SCHEMA_V1,
            "producer": { "name": "Stencila", "version": "9.9.9" },
            "asset": { "mediaType": "image/png", "digest": "sha256:abc" }
        });

        let parsed: ProvenanceAssertion = serde_json::from_value(raw).expect("deserialize");
        assert_eq!(parsed.asset.source_digest, "sha256:abc");
    }

    /// Ensures schema URLs outside the current v1 URL are reported as unknown.
    #[test]
    fn unknown_schema_url_detected() {
        let mut a = ProvenanceAssertion::new_v1("image/png", "sha256:abc");
        a.schema = "https://stencila.org/v999/ProvenanceAssertion.schema.json".to_string();
        assert!(!a.is_known_schema());
    }
}
