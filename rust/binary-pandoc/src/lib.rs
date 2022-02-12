use binary::{
    async_trait::async_trait,
    binary_clone_box,
    eyre::{bail, Result},
    semver_versions_matching, Binary, BinaryTrait,
};

pub struct PandocBinary;

#[async_trait]
impl BinaryTrait for PandocBinary {
    fn spec(&self) -> Binary {
        Binary::new("pandoc", &[], &["C:\\Users\\*\\AppData\\Local\\Pandoc"])
    }

    binary_clone_box!();

    async fn versions(&self, _os: &str) -> Result<Vec<String>> {
        self.versions_github_releases("jgm", "pandoc")
            .await
            .map(|versions| semver_versions_matching(versions, ">=2.14"))
    }

    async fn install_version(&self, version: &str, os: &str, arch: &str) -> Result<()> {
        // Map standard semver triples to Pandoc's version numbers (if they differ).
        // See https://github.com/jgm/pandoc/releases for mappings.
        let version = match version {
            "2.17.1" => "2.17.1.1",
            "2.17.0" => "2.17.0.1",
            "2.16.0" => "2.16",
            "2.15.0" => "2.15",
            "2.14.0" => "2.14.0.3",
            _ => version,
        };

        let url = format!(
            "https://github.com/jgm/pandoc/releases/download/{version}/pandoc-{version}-",
            version = version
        ) + match os {
            "macos" => "macOS.zip",
            "windows" => "windows-x86_64.zip",
            "linux" => match arch {
                "arm" => "linux-arm64.tar.gz",
                _ => "linux-amd64.tar.gz",
            },
            _ => bail!("Unable to determine Pandoc download URL"),
        };
        let archive = self.download(&url, None, None).await?;

        let dest = self.dir(Some(version.into()), true)?;
        self.extract(&archive, 1, &dest)?;
        self.executables(&dest, &["bin/pandoc", "pandoc.exe"])?;

        Ok(())
    }
}
