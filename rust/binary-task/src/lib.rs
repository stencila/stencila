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

pub struct TaskBinary;

#[async_trait]
impl BinaryTrait for TaskBinary {
    fn spec(&self) -> Binary {
        Binary::new("task", &[], &[])
    }

    binary_clone_box!();

    async fn versions(&self, _os: &str) -> Result<Vec<String>> {
        let versions = self.versions_update_maybe(
            versions::VERSIONS,
            self.versions_github_releases("go-task", "task").await,
        );
        Ok(self.semver_versions_matching(&versions, ">=3"))
    }

    async fn install_version(
        &self,
        version: &str,
        dest: &Path,
        os: &str,
        arch: &str,
    ) -> Result<()> {
        let url = format!(
            "https://github.com/go-task/task/releases/download/v{version}/task_",
            version = version
        ) + match os {
            "linux" => match arch {
                "arm" => "linux_arm64.tar.gz",
                "x86" => "linux_386.tar.gz",
                _ => "linux_amd64.tar.gz",
            },
            "macos" => match arch {
                "arm" => "darwin_arm64.tar.gz",
                _ => "darwin_amd64.tar.gz",
            },
            "windows" => match arch {
                "arm" => "windows_arm64.zip",
                "x86" => "windows_386.zip",
                _ => "windows_amd64.zip",
            },
            _ => bail!(
                "Installation of `task` for operating system `{}` is not supported",
                os
            ),
        };
        let archive = self.download(&url, None, None).await?;

        self.extract(&archive, dest, 0)?;
        self.executables(dest, &["pack"])?;

        Ok(())
    }
}
