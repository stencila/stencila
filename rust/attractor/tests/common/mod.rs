//! Shared test utilities for the attractor crate.

use stencila_attractor::error::{AttractorError, AttractorResult};

/// Create a temporary directory, mapping the error to [`AttractorError::Io`].
pub fn make_tempdir() -> AttractorResult<tempfile::TempDir> {
    tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })
}
