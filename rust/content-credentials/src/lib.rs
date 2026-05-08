//! C2PA Content Credentials for Stencila.
//!
//! This crate wraps the [`c2pa`] SDK to produce and verify
//! C2PA Content Credentials for Stencila assets, with a custom
//! `org.stencila.provenance` assertion.
//!
//! The MVP exposes the underlying C2PA mechanics — sign, embed-or-sidecar,
//! verify with a four-status report — without committing to a final shape
//! for the assertion payload. The payload is intentionally minimal here;
//! richer fields land in a follow-up phase and are designed to be additive.

#![warn(clippy::pedantic)]

pub mod assertion;
pub mod error;
pub mod producer;
pub mod report;
pub mod schema;
pub mod signer;
pub mod trust;
pub mod verifier;

pub mod media;

#[cfg(feature = "cli")]
pub mod cli;

pub use assertion::ProvenanceAssertion;
pub use error::{Error, Result};
pub use producer::{CredentialProducer, ManifestKind, SignAssetRequest, SignedAsset};
pub use report::{
    AssetBindingStatus, ManifestStatus, ProvenanceStatus, ReproducibilityStatus, SignerStatus,
    VerificationReport,
};
pub use schema::{PROVENANCE_LABEL, PROVENANCE_SCHEMA_V1};
pub use signer::{CredentialSignerConfig, init_dev_cert};
pub use trust::{TrustListStatus, official_trust_anchors, refresh_official_trust_list};
pub use verifier::{CredentialVerifier, VerifyAssetRequest};
