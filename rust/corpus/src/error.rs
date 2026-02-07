use std::path::PathBuf;

/// Errors that can occur in the corpus crate.
#[derive(Debug)]
pub enum Error {
    /// A segment is sealed and cannot be written to.
    SegmentSealed(u64),
    /// A segment file was not found at the expected path.
    SegmentNotFound(PathBuf),
    /// Manifest file could not be loaded.
    ManifestLoad(PathBuf, String),
    /// Manifest file could not be saved.
    ManifestSave(PathBuf, String),
    /// State database error.
    State(String),
    /// SQLite error.
    Sqlite(rusqlite::Error),
    /// IO error.
    Io(std::io::Error),
    /// JSON serialization/deserialization error.
    Json(serde_json::Error),
    /// Generic error message.
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SegmentSealed(id) => write!(f, "segment {id} is sealed and read-only"),
            Error::SegmentNotFound(p) => write!(f, "segment file not found: {}", p.display()),
            Error::ManifestLoad(p, e) => {
                write!(f, "failed to load manifest from {}: {e}", p.display())
            }
            Error::ManifestSave(p, e) => {
                write!(f, "failed to save manifest to {}: {e}", p.display())
            }
            Error::State(e) => write!(f, "state db error: {e}"),
            Error::Sqlite(e) => write!(f, "sqlite error: {e}"),
            Error::Io(e) => write!(f, "io error: {e}"),
            Error::Json(e) => write!(f, "json error: {e}"),
            Error::Other(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Error::Sqlite(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, Error>;
