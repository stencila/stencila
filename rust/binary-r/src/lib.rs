use std::path::Path;

use binary::{
    async_trait::async_trait,
    binary_clone_box,
    eyre::{bail, Result},
    Binary,
};
pub use binary::{BinaryInstallation, BinaryTrait};
use binary_asdf::AsdfBinary;

mod versions;

pub struct RBinary;

#[async_trait]
impl BinaryTrait for RBinary {
    fn spec(&self) -> Binary {
        Binary::new("R", &[], &["C:\\Program Files\\R\\R-*\\bin"])
    }

    binary_clone_box!();

    async fn versions(&self, os: &str) -> Result<Vec<String>> {
        let versions = if os == "linux" || os == "macos" {
            let versions = AsdfBinary::list_all("R").await?;
            self.semver_versions_matching(versions, "*")
        } else {
            versions::VERSIONS
                .iter()
                .map(|str| str.to_string())
                .collect()
        };
        Ok(versions)
    }

    async fn install_version(
        &self,
        version: &str,
        _dest: &Path,
        os: &str,
        _arch: &str,
    ) -> Result<()> {
        if os == "linux" || os == "macos" {
            let asdf = AsdfBinary {}.ensure().await?;
            asdf.run(&["plugin", "add", "R"]).await?;
            asdf.run(&["install", "R", version]).await?;
            Ok(())
        } else {
            bail!("Installation of R on Windows is not yet supported")
        }
    }
}
