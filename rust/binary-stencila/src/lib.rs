use std::path::Path;

pub use binary::BinaryTrait;
use binary::{
    async_trait::async_trait,
    binary_clone_box,
    eyre::{bail, Result},
    Binary,
};

pub struct StencilaBinary;

#[async_trait]
impl BinaryTrait for StencilaBinary {
    fn spec(&self) -> Binary {
        Binary::new("stencila", &[], &[])
    }

    binary_clone_box!();

    async fn versions(&self, _os: &str) -> Result<Vec<String>> {
        self.versions_github_releases("stencila", "stencila")
            .await
            .map(|versions| self.semver_versions_matching(versions, ">=1"))
    }

    async fn install_version(
        &self,
        version: &str,
        dest: &Path,
        os: &str,
        _arch: &str,
    ) -> Result<()> {
        let suffix = match os {
            "linux" => "x86_64-unknown-linux-gnu.tar.gz",
            "macos" => "x86_64-apple-darwin.tar.gz",
            "windows" => "x86_64-pc-windows-msvc.zip",
            _ => bail!(
                "Installation of `stencila` for operating system `{}` is not supported",
                os
            ),
        };
        let url = format!(
            "https://github.com/stencila/stencila/releases/download/v{version}/stencila-v{version}-",
            version = version
        ) + suffix;
        let archive = self.download(&url, None, None).await?;

        self.extract(&archive, 0, dest)?;
        self.executables(dest, &["stencila"])?;

        Ok(())
    }
}
