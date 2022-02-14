use std::{env, ffi::OsString, fs};

use binary::{
    async_trait::async_trait,
    binaries_dir, binary_clone_box,
    eyre::{bail, Result},
    semver_versions_sorted, Binary, BinaryTrait,
};

/// A `BinaryTrait` for `asdf`
///
/// This sets the `$ASDF_DATA_DIR` to be the same as the Stencila `binaries` directory.
/// Given that `asdf` uses the same structure for this directory as Stencila, it means that
/// executables that are installed by `asdf` have the same expected location as those installed
/// by Stencila.
pub struct AsdfBinary;

#[async_trait]
impl BinaryTrait for AsdfBinary {
    fn spec(&self) -> Binary {
        Binary::new("asdf", &[], &[])
    }

    binary_clone_box!();

    async fn versions(&self, _os: &str) -> Result<Vec<String>> {
        self.versions_github_releases("asdf-vm", "asdf")
            .await
            .map(semver_versions_sorted)
    }

    fn run_env(&self, version: Option<String>) -> Vec<(String, OsString)> {
        if let Ok(dir) = self.dir(version, false) {
            let binaries_dir = binaries_dir();
            if let Ok(..) = dir.strip_prefix(&binaries_dir) {
                let asdf_config_file = dir.join(".asdfrc").into_os_string();
                let asdf_dir = dir.into_os_string();
                let asdf_data_dir = binaries_dir.into_os_string();
                let path = OsString::from(env::var("PATH").unwrap_or_default());
                let path = env::join_paths(&[asdf_dir.clone(), path.clone()]).unwrap_or(path);
                return vec![
                    ("ASDF_CONFIG_FILE".to_string(), asdf_config_file),
                    ("ASDF_DIR".to_string(), asdf_dir),
                    ("ASDF_DATA_DIR".to_string(), asdf_data_dir),
                    ("PATH".to_string(), path),
                ];
            }
        }
        Vec::new()
    }

    async fn install_version(&self, version: &str, os: &str, _arch: &str) -> Result<()> {
        if os == "windows" {
            bail!("`asdf` can not be install on Windows")
        }

        let url = format!(
            "https://github.com/asdf-vm/asdf/archive/refs/tags/v{version}.tar.gz",
            version = version
        );
        let filename = format!("asdf-v{version}.tar.gz", version = version);
        let archive = self.download(&url, Some(filename), None).await?;

        let dest = self.dir(Some(version.into()), true)?;
        self.extract(&archive, 1, &dest)?;
        self.executables(&dest, &["bin/asdf"])?;

        // TODO: use a setting to determine the keep downloads policy for both Stencila and asdf
        fs::write(dest.join(".asdfrc"), "always_keep_download = yes\n")?;

        Ok(())
    }
}

impl AsdfBinary {
    /// List all versions for an `asdf` plugin
    pub async fn list_all(plugin: &str) -> Result<Vec<String>> {
        let asdf = AsdfBinary {}.require(None, true).await?;
        asdf.run(&["plugin", "add", plugin]).await?;
        let output = asdf.run(&["list", "all", plugin]).await?;

        let versions: Vec<String> = std::str::from_utf8(&output.stdout)?
            .lines()
            .map(String::from)
            .collect();

        Ok(versions)
    }

    /// Install a version for an `asdf` plugin
    pub async fn install(plugin: &str, version: &str) -> Result<()> {
        let asdf = AsdfBinary {}.require(None, true).await?;
        asdf.run(&["plugin", "add", plugin]).await?;
        asdf.run(&["install", plugin, version]).await?;
        Ok(())
    }
}
