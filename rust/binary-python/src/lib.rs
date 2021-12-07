use binary::{
    async_trait::async_trait,
    eyre::{bail, Result},
    Binary, BinaryTrait,
};

pub struct PythonBinary {}

#[async_trait]
impl BinaryTrait for PythonBinary {
    #[rustfmt::skip]
    fn spec(&self) -> Binary {
        Binary::new(
            "python",
            &["python3"],
            &["C:/Python3*"],
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
        // On Windows uses Pythons "embeddable" distributions intended for this purpose.
        let url = format!(
            "https://www.python.org/ftp/python/{version}/python-{version}-embed-",
            version = version
        ) + match os {
            "windows" => match arch {
                "x86" => "win32.zip",
                "x86_64" => "amd64.zip",
                _ => bail!("Unhandled arch '{}", arch),
            },
            _ => bail!(
                "Sorry, I'm unable to install Python for operating system `{}`. Perhaps install it manually?",
                os
            ),
        };

        let archive = self.download(&url).await?;
        let dest = self.dir(Some(version.into()), true)?;
        self.extract(&archive, 0, &dest)?;
        self.executable(&dest, &["bin/python3", "python3.exe"])?;

        Ok(())
    }
}
