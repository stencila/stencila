use std::ffi::OsString;

use binary::{
    async_trait::async_trait, binary_clone_box, eyre::Result, semver_versions_matching, tracing,
    Binary, BinaryTrait,
};
use binary_python::PythonBinary;

/// A `BinaryTrait` for `poetry`
pub struct PoetryBinary;

#[async_trait]
impl BinaryTrait for PoetryBinary {
    fn spec(&self) -> Binary {
        Binary::new("poetry", &[], &[])
    }

    binary_clone_box!();

    async fn versions(&self, _os: &str) -> Result<Vec<String>> {
        self.versions_github_releases("python-poetry", "poetry")
            .await
            // "installer does not support Poetry releases < 0.12.0"
            .map(|versions| semver_versions_matching(versions, ">=0.12"))
    }

    async fn install_version(&self, version: &str, _os: &str, _arch: &str) -> Result<()> {
        let script = self
            .download(
                "https://install.python-poetry.org",
                Some("install-poetry.py".to_string()),
                None,
            )
            .await?;
        let home = self.dir(Some(version.to_string()), true)?;

        tracing::info!("Running install-poetry.py");
        let mut python = PythonBinary {}
            .require(Some(">=3.6".to_string()), true)
            .await?;
        python.envs(&[
            ("POETRY_HOME", home.into_os_string()),
            ("POETRY_VERSION", OsString::from(version)),
            ("POETRY_ACCEPT", OsString::from("yes")),
        ]);
        let _output = python.run(&[&script.display().to_string()]).await?;

        Ok(())
    }
}
