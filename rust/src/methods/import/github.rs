use std::path::Path;

use crate::{files::File, sources::GitHub};
use eyre::Result;

/// Import files from a GitHub repository into a project
pub async fn import(project: &Path, source: &GitHub, destination: Option<String>) -> Result<Vec<File>> {
    // TODO
    Ok(vec![])
}
