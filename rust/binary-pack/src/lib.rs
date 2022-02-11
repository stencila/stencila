pub use binary::BinaryTrait;
use binary::{
    async_trait::async_trait,
    binary_clone_box,
    eyre::{bail, Result},
    Binary,
};

pub struct PackBinary;

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

    binary_clone_box!();

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
            _ => bail!(
                "Installation of `pack` for operating system `{}` is not supported",
                os
            ),
        };
        let archive = self.download(&url, None, None).await?;

        let dest = self.dir(Some(version.into()), true)?;
        self.extract(&archive, 0, &dest)?;
        self.executables(&dest, &["pack"])?;

        Ok(())
    }
}
