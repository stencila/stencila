use binary::{
    async_trait::async_trait,
    eyre::{bail, Result},
    Binary, BinaryTrait,
};

pub struct NodeBinary {}

#[async_trait]
impl BinaryTrait for NodeBinary {
    #[rustfmt::skip]
    fn spec(&self) -> Binary {
        Binary::new(
            "node",
            &[],
            &["C:\\Program Files\\nodejs"],
            // Release list at https://nodejs.org/en/download/releases/
            // Current strategy is to support the latest patch version of each minor version.
            // Support for older minor versions may be progressively  dropped if there are no
            // plugins relying on them.
            &[
                "16.10.0",
                "16.11.1",
                "16.12.0",
                "16.13.1",
                "17.0.1",
                "17.1.0",
                "17.2.0"
            ],
        )
    }

    async fn install_version(&self, version: &str, os: &str, arch: &str) -> Result<()> {
        let url = format!(
            "https://nodejs.org/dist/v{version}/node-v{version}-",
            version = version
        ) + match os {
            "macos" => match arch {
                "arm" => "darwin-arm64.tar.gz",
                _ => "darwin-x64.tar.gz",
            },
            "windows" => match arch {
                "x86" => "win-x86.zip",
                _ => "win-x64.zip",
            },
            "linux" => match arch {
                "arm" => "linux-arm64.tar.xz",
                _ => "linux-x64.tar.xz",
            },
            _ => bail!("Unable to determine Node download URL"),
        };

        let archive = self.download(&url).await?;
        let dest = self.dir(Some(version.into()), true)?;
        self.extract(&archive, 1, &dest)?;
        self.executable(&dest, &["bin/node", "bin/npm", "node.exe", "npm"])?;

        Ok(())
    }
}
