use binary::{
    async_trait::async_trait,
    eyre::{bail, Result},
    Binary, BinaryTrait,
};
use binary_asdf::AsdfBinary;

pub struct PythonBinary;

#[async_trait]
impl BinaryTrait for PythonBinary {
    #[rustfmt::skip]
    fn spec(&self) -> Binary {
        Binary::new(
            "python",
            &["python3"],
            &["C:\\Python3*"],
            // Release list at https://www.python.org/downloads/.
            // Current strategy is to support the latest patch version of each minor version.
            &[
                "3.8.12",
                "3.9.9",
                "3.10.0"
            ],
        )
    }

    async fn install_version(&self, version: &str, os: &str, arch: &str) -> Result<()> {
        // On Linux or Mac use `asdf` to install
        if os == "linux" || os == "macos" {
            let asdf = AsdfBinary {}.require(None, true).await?;
            asdf.run(&["plugin", "add", "python"]).await?;
            asdf.run(&["install", "python", version]).await?;
            return Ok(());
        }

        // On Windows uses Pythons "embeddable" distributions intended for this purpose.
        let url = format!(
            "https://www.python.org/ftp/python/{version}/python-{version}-embed-",
            version = version
        ) + match arch {
            "x86" => "win32.zip",
            "x86_64" => "amd64.zip",
            _ => bail!("Unhandled arch '{}", arch),
        };
        let archive = self.download(&url, None, None).await?;

        let dest = self.dir(Some(version.into()), true)?;
        self.extract(&archive, 0, &dest)?;
        self.executables(&dest, &["bin/python3", "python3.exe"])?;

        Ok(())
    }
}
