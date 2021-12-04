use binary::{
    async_trait::async_trait,
    eyre::{bail, Result},
    Binary, BinaryTrait,
};

pub struct PandocBinary {}

#[async_trait]
impl BinaryTrait for PandocBinary {
    fn spec(&self) -> Binary {
        Binary::new(
            "pandoc",
            &[],
            // Release list at https://github.com/jgm/pandoc/releases
            // To avoid version parsing issues we map standard semver triples
            // to Pandoc's quads in the `install_pandoc` function and use only triples here.
            &["2.14.0", "2.14.1", "2.14.2", "2.15.0", "2.16.0"],
        )
    }

    async fn install_version(&self, version: &str, os: &str, arch: &str) -> Result<()> {
        // Map standard semver triples to Pandoc's version numbers
        // See https://github.com/jgm/pandoc/releases
        let version = match version {
            "2.14.0" => "2.14.0.3",
            "2.15.0" => "2.15",
            "2.16.0" => "2.16",
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

        let archive = self.download(&url).await?;
        let dest = self.dir(Some(version.into()), true)?;
        self.extract(&archive, 1, &dest)?;
        self.executable(&dest, &["bin/pandoc", "pandoc.exe"])?;

        Ok(())
    }
}
