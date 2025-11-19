use std::path::PathBuf;

use url::Url;

/// Options for a dry-run of a push
#[derive(Debug, Clone)]
pub struct PushDryRunOptions {
    /// Whether dry-run is enabled
    pub enabled: bool,

    /// Optional output directory for generated files
    pub output_dir: Option<PathBuf>,
}

/// Information about a file generated during a push dry-run
#[derive(Debug, Clone)]
pub struct PushDryRunFile {
    /// Storage path in R2 (e.g., "siteid/branch/report/index.html.gz")
    pub storage_path: String,

    /// Local path where file was written (if dry-run with output dir)
    pub local_path: Option<PathBuf>,

    /// File size in bytes
    pub size: u64,

    /// Whether the file is compressed
    pub compressed: bool,

    /// Route this file serves (for HTML files)
    pub route: Option<String>,
}

/// Result of a push operation
#[derive(Debug)]
pub enum PushResult {
    /// Files were uploaded to the site
    Uploaded(Url),

    /// Dry-run completed without uploading
    DryRun {
        /// The URL where files would be published
        url: Url,

        /// List of files that would be uploaded
        files: Vec<PushDryRunFile>,

        /// Directory where files were saved (if specified)
        output_dir: Option<PathBuf>,
    },
}

impl PushResult {
    pub fn url(&self) -> Url {
        match self {
            PushResult::Uploaded(url) => url,
            PushResult::DryRun { url, .. } => url,
        }
        .clone()
    }
}
