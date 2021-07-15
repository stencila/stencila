use std::path::Path;

use crate::{files::File, sources::GitHub};
use eyre::Result;

/// Import from a GitHub repository
pub fn import(_project: &Path, _source: &GitHub, _destination: Option<String>) -> Result<Vec<File>> {
    Ok(vec![])
}
