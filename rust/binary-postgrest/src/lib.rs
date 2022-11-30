use std::path::Path;

pub use binary::BinaryTrait;
use binary::{
    binary_clone_box,
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
    },
    Binary,
};

mod versions;

pub struct PostgrestBinary;

#[async_trait]
impl BinaryTrait for PostgrestBinary {
    fn spec(&self) -> Binary {
        Binary::new("postgrest", &[], &[])
    }

    binary_clone_box!();

    async fn versions(&self, _os: &str) -> Result<Vec<String>> {
        let versions = self.versions_update_maybe(
            versions::VERSIONS,
            self.versions_github_releases("PostgREST", "postgrest")
                .await,
        );
        Ok(self.semver_versions_matching(&versions, ">=10"))
    }

    async fn install_version(
        &self,
        version: &str,
        dest: &Path,
        os: &str,
        _arch: &str,
    ) -> Result<()> {
        let url = format!(
            "https://github.com/PostgREST/postgrest/releases/download/v{version}/postgrest-v{version}-",
            version = version
        ) + match os {
            "linux" => "linux-static-x64.tar.xz",
            "macos" => "macos-x64.tar.xz",
            "windows" => "windows-x64.zip",
            _ => bail!(
                "Installation of `postgrest` for operating system `{}` is not supported",
                os
            ),
        };
        let archive = self.download(&url, None, None).await?;

        self.extract(&archive, dest, 0)?;
        self.executables(dest, &["postgrest"])?;

        Ok(())
    }
}
