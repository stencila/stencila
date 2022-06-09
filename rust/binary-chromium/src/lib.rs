use std::path::Path;

use binary::{
    binary_clone_box,
    common::{async_trait::async_trait, eyre::Result, tracing},
    Binary, BinaryTrait,
};
use binary_chrome::ChromeBinary;

mod versions;

pub struct ChromiumBinary;

const ORG: &str = "macchrome";
const REPO: &str = "linchrome";

#[async_trait]
impl BinaryTrait for ChromiumBinary {
    fn spec(&self) -> Binary {
        Binary::new(
            "chromium",
            &[],
            &[
                "/Applications/Chromium.app/Contents/MacOS",
                "C:\\Program Files\\Chromium\\Application",
            ],
        )
    }

    binary_clone_box!();

    async fn versions(&self, _os: &str) -> Result<Vec<String>> {
        let versions = self
            .versions_github_releases(ORG, REPO)
            .await
            .map(|versions| {
                versions
                    .iter()
                    .map(|version| {
                        let mut parts = version
                            .strip_suffix("-portable-ungoogled-Lin64")
                            .unwrap_or(version)
                            .splitn(4, '.');
                        format!(
                            "{}.{}.{}+{}",
                            parts.next().unwrap_or_default(),
                            parts.next().unwrap_or_default(),
                            parts.next().unwrap_or_default(),
                            parts.next().unwrap_or_default()
                        )
                    })
                    .collect()
            });

        let versions = self.versions_update_maybe(versions::VERSIONS, versions);

        // Prior to v87 (Jan 2021) tar.xz files had different names so `install_version` will not work
        let versions = self.semver_versions_matching(&versions, ">=87");

        Ok(versions)
    }

    async fn install_version(
        &self,
        version: &str,
        dest: &Path,
        os: &str,
        arch: &str,
    ) -> Result<()> {
        if os != "linux" && arch != "amd64" {
            tracing::warn!("Installation of Chromium is not supported for this OS and/or architecture; falling back to installing Chrome");
            return ChromeBinary {}
                .install_version(version, dest, os, arch)
                .await;
        }

        let version = self.semver_version(version)?;

        let tag = format!(
            "v{}.{}.{}.{}-portable-ungoogled-Lin64",
            version.major, version.minor, version.patch, version.build
        );
        let file = format!(
            "ungoogled-chromium_{}.{}.{}.{}_1.vaapi_linux.tar.xz",
            version.major,
            version.minor,
            version.patch,
            version.build.split('-').next().unwrap_or_default()
        );
        let url = format!("https://github.com/{ORG}/{REPO}/releases/download/{tag}/{file}");
        let archive = self.download(&url, None, None).await?;

        self.extract(&archive, dest, 1)?;
        self.executables(dest, &["chrome"])?;

        Ok(())
    }
}
