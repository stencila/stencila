//! Crate-local error type.

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("input asset not found: {0}")]
    InputNotFound(PathBuf),

    #[error("could not determine media type for: {0}")]
    UnknownMediaType(PathBuf),

    #[error("signing certificate not found at: {0}")]
    CertNotFound(PathBuf),

    #[error("signing key not found at: {0}")]
    KeyNotFound(PathBuf),

    #[error("both --cert and --key must be supplied together")]
    SignerOverridesIncomplete,

    #[error("both {0} and {1} must be set")]
    SignerEnvIncomplete(&'static str, &'static str),

    #[error("sidecar path conflicts with output asset path: {0}")]
    OutputSidecarConflict(PathBuf),

    #[error(
        "output media type for {output_path} ({output_media_type}) does not match input media type for {input_path} ({input_media_type})"
    )]
    OutputMediaTypeMismatch {
        input_path: PathBuf,
        input_media_type: String,
        output_path: PathBuf,
        output_media_type: String,
    },

    #[error(
        "no signing identity configured; pass --cert/--key, set STENCILA_CREDENTIALS_CERT/KEY, or run `stencila credentials init`"
    )]
    NoSignerConfigured,

    #[error("c2pa SDK error: {0}")]
    C2pa(#[from] c2pa::Error),

    #[error("pdf error: {0}")]
    Pdf(#[from] lopdf::Error),

    #[error("certificate generation error: {0}")]
    Rcgen(#[from] rcgen::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("task join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("{0}")]
    Other(String),
}

impl Error {
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other(message.into())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
