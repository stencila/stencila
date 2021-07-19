use std::path::Path;

use crate::{files::File, sources::GitHub};
use eyre::Result;

/// Import files from a GitHub repository into a project
pub async fn import(_project: &Path, _source: &GitHub, _destination: Option<String>) -> Result<Vec<File>> {
    todo!()
}
