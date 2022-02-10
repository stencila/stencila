use binary::{
    async_trait::async_trait,
    eyre::{bail, Result},
    Binary, BinaryTrait,
};

pub struct PandocBinary {}

#[async_trait]
impl BinaryTrait for PandocBinary {
    #[rustfmt::skip]
    fn spec(&self) -> Binary {
        Binary::new(
            "pandoc",
            &[],
            &["C:\\Users\\*\\AppData\\Local\\Pandoc"],
            // Release list at https://github.com/jgm/pandoc/releases.
            // Current strategy is to support the latest patch version of each minor version.
            //
            // Note: To avoid version parsing issues we map standard semver triples
            // to Pandoc's quads in the `install_pandoc` function and use only triples here.
            //
            // Note: See the documentation for the `PANDOC_SEMVER` variable in the `codec-pandoc`
            // sibling crate.
            &[
                "2.14.2",
                "2.15.0",
                "2.16.2"
            ],
        )
    }

    async fn install_version(&self, version: &str, os: &str, arch: &str) -> Result<()> {
        // Map standard semver triples to Pandoc's version numbers (if they differ).
        // See https://github.com/jgm/pandoc/releases for mappings.
        let version = match version {
            "2.15.0" => "2.15",
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
