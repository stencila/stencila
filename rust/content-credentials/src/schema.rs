//! Stable identifiers for the Stencila C2PA assertion.

/// C2PA assertion label for Stencila provenance.
///
/// Stable across normal evolution. Bumped only on a true wire-format break.
pub const PROVENANCE_LABEL: &str = "org.stencila.provenance";

/// Payload schema URL for v1 of the Stencila provenance assertion.
///
/// This follows the same shape as Stencila document JSON Schema URLs:
/// `https://stencila.org/v.../{Type}.schema.json`. Here `v1` is the
/// provenance assertion payload version, not the Stencila release version.
///
/// New optional fields and refined semantics mint a new schema URL without
/// changing [`PROVENANCE_LABEL`].
pub const PROVENANCE_SCHEMA_V1: &str =
    "https://stencila.org/c2pa/v1/ProvenanceAssertion.schema.json";
