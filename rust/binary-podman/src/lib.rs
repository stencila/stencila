use std::path::Path;

pub use binary::BinaryTrait;
use binary::{
    async_trait::async_trait,
    binary_clone_box,
    eyre::{bail, Result},
    Binary,
};

mod versions;

pub struct PodmanBinary;

#[async_trait]
impl BinaryTrait for PodmanBinary {
    fn spec(&self) -> Binary {
        Binary::new("podman", &[], &[])
    }

    binary_clone_box!();

    async fn versions(&self, _os: &str) -> Result<Vec<String>> {
        let versions = self.versions_update_maybe(
            versions::VERSIONS,
            self.versions_github_releases("containers", "podman").await,
        );
        Ok(self.semver_versions_matching(&versions, ">=3"))
    }

    async fn install_version(
        &self,
        version: &str,
        dest: &Path,
        os: &str,
        _arch: &str,
    ) -> Result<()> {
        let suffix = match os {
            "linux" => "static.tar.gz",
            "macos" => "release-darwin.zip",
            "windows" => "release-windows.zip",
            _ => bail!(
                "Installation of `podman` for operating system `{}` is not supported",
                os
            ),
        };
        let url = format!(
            "https://github.com/containers/podman/releases/download/v{version}/podman-remote-",
            version = version
        ) + suffix;
        let filename = ["podman-remote-", version, "-", suffix].concat();
        let archive = self.download(&url, Some(filename), None).await?;

        self.extract(&archive, dest, 1)?;
        self.executables(dest, &["podman"])?;

        Ok(())
    }
}
