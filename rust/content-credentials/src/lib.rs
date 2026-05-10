//! C2PA Content Credentials for Stencila.
//!
//! This crate wraps the [`c2pa`] SDK to produce and verify
//! C2PA Content Credentials for Stencila assets, with a custom
//! `org.stencila.provenance` assertion.
//!
//! The crate exposes the underlying C2PA mechanics — sign, embed-or-sidecar,
//! verify with a four-status report — and maps Stencila provenance snapshots
//! into a versioned `org.stencila.provenance` assertion payload.

#![warn(clippy::pedantic)]

pub mod assertion;
pub mod error;
pub mod policy;
pub mod producer;
pub mod report;
pub mod schema;
pub mod signer;
pub mod snapshot;
pub mod trust;
pub mod verifier;

pub mod media;

#[cfg(feature = "cli")]
pub mod cli;

pub use error::{Error, Result};
pub use policy::{CredentialProfile, ProjectionPolicy};
pub use producer::{CredentialProducer, ManifestKind, SignAssetRequest, SignedAsset};
pub use report::{
    AssetBindingStatus, ManifestStatus, ProvenanceStatus, ReproducibilityStatus, SignerStatus,
    VerificationReport,
};
pub use schema::{PROVENANCE_LABEL, PROVENANCE_SCHEMA, ProvenanceAssertion};
pub use signer::{CredentialSignerConfig, init_dev_cert};
pub use snapshot::{
    ActivitySnapshot, AgentSnapshot, AiDisclosureSnapshot, AssetSnapshot, AttributionSnapshot,
    DefinitionSnapshot, DependencySnapshot, DisclosureAssessmentSnapshot, DocumentSnapshot,
    EnvironmentSnapshot, ExecutionDigestSnapshot, ExecutionMessageSnapshot, ExecutionSnapshot,
    FileDigestSnapshot, IdentifierSnapshot, KernelSnapshot, PrivacySnapshot, ProducerSnapshot,
    ProvenanceCategorySnapshot, ProvenanceSnapshot, ProvenanceSummarySnapshot, RedactionSnapshot,
    ReproducibilitySnapshot, RuntimeSnapshot, SourceSnapshot, WorkflowSnapshot,
};
pub use trust::{TrustListStatus, official_trust_anchors, refresh_official_trust_list};
pub use verifier::{CredentialVerifier, VerifyAssetRequest};
