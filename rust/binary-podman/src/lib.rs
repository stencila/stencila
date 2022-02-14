pub use binary::BinaryTrait;
use binary::{
    async_trait::async_trait,
    binary_clone_box,
    eyre::{bail, Result},
    semver_versions_matching, Binary,
};

pub struct PodmanBinary;

#[async_trait]
impl BinaryTrait for PodmanBinary {
    fn spec(&self) -> Binary {
        Binary::new("podman", &[], &[])
    }

    binary_clone_box!();

    async fn versions(&self, _os: &str) -> Result<Vec<String>> {
        self.versions_github_releases("containers", "podman")
            .await
            .map(|versions| semver_versions_matching(versions, ">=3"))
    }

    async fn install_version(&self, version: &str, os: &str, _arch: &str) -> Result<()> {
        let url = format!(
            "https://github.com/containers/podman/releases/download/v{version}/podman-remote-",
            version = version
        ) + match os {
            "linux" => "static.tar.gz",
            "macos" => "release-darwin.zip",
            "windows" => "release-windows.zip",
            _ => bail!(
                "Installation of `podman` for operating system `{}` is not supported",
                os
            ),
        };
        let archive = self.download(&url, None, None).await?;

        let dest = self.dir(Some(version.into()), true)?;
        self.extract(&archive, 0, &dest)?;
        self.executables(&dest, &["podman"])?;

        Ok(())
    }
}
