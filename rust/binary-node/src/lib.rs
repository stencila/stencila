use std::path::Path;

use binary::{
    async_trait::async_trait,
    binary_clone_box,
    eyre::{bail, Result},
};
pub use binary::{Binary, BinaryInstallation, BinaryTrait};

mod versions;

pub struct NodeBinary;

#[async_trait]
impl BinaryTrait for NodeBinary {
    #[rustfmt::skip]
    fn spec(&self) -> Binary {
        Binary::new(
            "node",
            &[],
            &["C:\\Program Files\\nodejs"]
        )
    }

    binary_clone_box!();

    async fn versions(&self, _os: &str) -> Result<Vec<String>> {
        Ok(self.versions_update_maybe(
            versions::VERSIONS,
            self.versions_github_releases("nodejs", "node").await,
        ))
    }

    async fn install_version(
        &self,
        version: &str,
        dest: &Path,
        os: &str,
        arch: &str,
    ) -> Result<()> {
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
        let archive = self.download(&url, None, None).await?;

        self.extract(&archive, dest, 1)?;
        self.executables(
            dest,
            &["bin/node", "bin/npm", "bin/npx", "node.exe", "npm", "npx"],
        )?;

        Ok(())
    }
}
