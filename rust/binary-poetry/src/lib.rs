use std::ffi::OsString;
use std::path::Path;

use binary::{
    binary_clone_box,
    common::{async_trait::async_trait, eyre::Result, tracing},
    Binary, BinaryTrait,
};
use binary_python::PythonBinary;

mod versions;

pub struct PoetryBinary;

#[async_trait]
impl BinaryTrait for PoetryBinary {
    fn spec(&self) -> Binary {
        Binary::new("poetry", &[], &[])
    }

    binary_clone_box!();

    async fn versions(&self, _os: &str) -> Result<Vec<String>> {
        let versions = self.versions_update_maybe(
            versions::VERSIONS,
            self.versions_github_releases("python-poetry", "poetry")
                .await,
        );
        // "installer does not support Poetry releases < 0.12.0"
        Ok(self.semver_versions_matching(&versions, ">=0.12"))
    }

    async fn install_version(
        &self,
        version: &str,
        dest: &Path,
        _os: &str,
        _arch: &str,
    ) -> Result<()> {
        let script = self
            .download(
                "https://install.python-poetry.org",
                Some("install-poetry.py".to_string()),
                None,
            )
            .await?;

        tracing::info!(
            "Running `install-poetry.py` to install Poetry `{}` to `{}`",
            version,
            dest.display()
        );
        let mut python = PythonBinary {}.ensure_version(">=3.7").await?;
        python.env_list(&[
            ("POETRY_HOME", dest.into()),
            ("POETRY_VERSION", version.into()),
            ("POETRY_ACCEPT", OsString::from("yes")),
        ]);
        python.run(&[&script.display().to_string()]).await?;

        Ok(())
    }
}
