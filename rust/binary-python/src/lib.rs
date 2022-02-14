use binary::{
    async_trait::async_trait,
    binary_clone_box,
    eyre::{bail, Result},
    semver_versions_matching,
};
pub use binary::{Binary, BinaryTrait};
use binary_asdf::AsdfBinary;

mod versions;
pub struct PythonBinary;

#[async_trait]
impl BinaryTrait for PythonBinary {
    fn spec(&self) -> Binary {
        Binary::new("python", &["python3"], &["C:\\Python3*"])
    }

    binary_clone_box!();

    async fn versions(&self, os: &str) -> Result<Vec<String>> {
        let versions = if os == "linux" || os == "macos" {
            let versions = AsdfBinary::list_all("python").await?;
            semver_versions_matching(versions, "*")
        } else {
            versions::VERSIONS
                .iter()
                .map(|str| str.to_string())
                .collect()
        };
        Ok(versions)
    }

    async fn install_version(&self, version: &str, os: &str, arch: &str) -> Result<()> {
        // On Linux or Mac use `asdf` to install
        if os == "linux" || os == "macos" {
            AsdfBinary::install("python", version).await
        } else {
            // On Windows uses Python's "embeddable" distributions intended for this purpose.
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
            self.executables(&dest, &["bin/python3", "python3.exe"])
        }
    }
}
