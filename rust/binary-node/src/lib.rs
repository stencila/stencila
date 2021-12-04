use binary::{
    async_trait::async_trait,
    eyre::{bail, Result},
    Binary, BinaryTrait,
};

pub struct NodeBinary {}

#[async_trait]
impl BinaryTrait for NodeBinary {
    fn spec(&self) -> Binary {
        Binary::new(
            "node",
            &[],
            // Release list at https://nodejs.org/en/download/releases/
            &[
                "16.4.0", "16.4.1", "16.4.2", "16.5.0", "16.6.0", "16.6.1", "16.6.2", "16.7.0",
                "16.8.0", "16.9.0", "16.9.1", "16.10.0", "16.11.0", "16.11.1", "16.12.0",
                "16.13.0", "17.0.0", "17.0.1",
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
