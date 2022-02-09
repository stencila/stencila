use binary::{
    async_trait::async_trait,
    eyre::{bail, Result},
    Binary, BinaryTrait,
};

pub struct PackBinary {}

#[async_trait]
impl BinaryTrait for PackBinary {
    #[rustfmt::skip]
    fn spec(&self) -> Binary {
        Binary::new(
            "pack",
            &[],
            &[],
            // Release list at https://github.com/buildpacks/pack/releases
            // Current strategy is to support the latest patch version of the last three minor version.
            &[
                "0.23.0",
                "0.22.0",
                "0.21.1"
            ],
        )
    }

    async fn install_version(&self, version: &str, os: &str, arch: &str) -> Result<()> {
        let url = format!(
            "https://github.com/buildpacks/pack/releases/download/v{version}/pack-v{version}-",
            version = version
        ) + match os {
            "linux" => "linux.tgz",
            "macos" => match arch {
                "arm" => "macos-arm64.tgz",
                _ => "macos.tgz",
            },
            "windows" => "windows.zip",
            _ => bail!("Unable to determine Node download URL"),
        };

        let archive = self.download(&url).await?;
        let dest = self.dir(Some(version.into()), true)?;
        self.extract(&archive, 0, &dest)?;
        self.executables(&dest, &["pack"])?;

        Ok(())
    }
}
