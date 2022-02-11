pub use binary;
use binary::{
    async_trait::async_trait,
    binary_clone_box,
    eyre::{bail, Result},
    Binary, BinaryTrait,
};

pub struct PodmanBinary;

#[async_trait]
impl BinaryTrait for PodmanBinary {
    #[rustfmt::skip]
    fn spec(&self) -> Binary {
        Binary::new(
            "podman",
            &[],
            &[],
            // Release list at https://github.com/containers/podman/releases
            &[
               "3.4.3"
            ],
        )
    }

    binary_clone_box!();

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
