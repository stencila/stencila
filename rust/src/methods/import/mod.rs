use std::path::Path;

use crate::{files::File, sources::Source};
use eyre::{Result};

#[cfg(feature = "import-elife")]
pub mod elife;
#[cfg(feature = "import-github")]
pub mod github;

/// Import files from a source into a project
///
/// # Arguments
///
/// - `project`: the path of the project being imported into
/// - `source`: the source to import from
/// - `destination`: the destination path within the project to import to
///
/// # Returns
///
/// A list of files imported from the source.
pub async fn import(project: &Path, source: &Source, destination: Option<String>) -> Result<Vec<File>> {
    // Allow these for when no features are enabled
    #[allow(unused_variables, unreachable_code)]
    let files = match source {
        #[cfg(feature = "import-elife")]
        Source::Elife(source) => elife::import(project, source, destination).await?,

        #[cfg(feature = "import-github")]
        Source::GitHub(source) => github::import(project, source, destination).await?,
    };
    Ok(files)
}
