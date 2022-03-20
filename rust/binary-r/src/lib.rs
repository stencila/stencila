use std::{collections::HashMap, path::Path};

use binary::{
    async_trait::async_trait,
    binary_clone_box,
    eyre::{bail, eyre, Result},
    http_utils,
};
pub use binary::{Binary, BinaryInstallation, BinaryTrait};
use binary_asdf::AsdfBinary;

mod versions;

pub struct RBinary;

#[async_trait]
impl BinaryTrait for RBinary {
    fn spec(&self) -> Binary {
        Binary::new("R", &[], &["C:\\Program Files\\R\\R-*\\bin"])
    }

    binary_clone_box!();

    async fn versions(&self, _os: &str) -> Result<Vec<String>> {
        async fn more() -> Result<Vec<String>> {
            // Previously we used `AsdfBinary::list_all("R").await?;` to get a list of
            // versions (and embedded a static copy of that list for Windows). This approach
            // has the advantage of not needing to `asdf` to be installed and being dynamically
            // updatable on Windows.
            let versions: HashMap<String, Vec<String>> =
                http_utils::get("https://cdn.rstudio.com/r/versions.json").await?;
            let versions = versions
                .get("r_versions")
                .cloned()
                .ok_or_else(|| eyre!("Expected object with `r_versions` property"))?;
            Ok(versions)
        }
        Ok(self.versions_update_maybe(versions::VERSIONS, more().await))
    }

    async fn install_version(
        &self,
        version: &str,
        _dest: &Path,
        os: &str,
        _arch: &str,
    ) -> Result<()> {
        if os == "linux" || os == "macos" {
            AsdfBinary::install("R", version).await
        } else {
            bail!("Installation of R on Windows is not yet supported")
        }
    }
}
