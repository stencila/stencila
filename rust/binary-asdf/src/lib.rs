use std::{
    env,
    ffi::{OsStr, OsString},
    fs,
    path::Path,
};

use binary::{
    async_trait::async_trait,
    binaries_dir, binary_clone_box,
    eyre::{bail, Result},
    Binary, BinaryTrait,
};

mod versions;

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
        Ok(self.versions_update_maybe(
            versions::VERSIONS,
            self.versions_github_releases("asdf-vm", "asdf").await,
        ))
    }

    fn run_env(&self, version: Option<String>) -> Vec<(OsString, OsString)> {
        if let Ok(dir) = self.dir(version, false) {
            let binaries_dir = binaries_dir();
            if let Ok(..) = dir.strip_prefix(&binaries_dir) {
                let asdf_config_file = dir.join(".asdfrc").into_os_string();
                let asdf_dir = dir.into_os_string();
                let asdf_data_dir = binaries_dir.into_os_string();
                let path = OsString::from(env::var("PATH").unwrap_or_default());
                let path = env::join_paths(&[asdf_dir.clone(), path.clone()]).unwrap_or(path);
                return vec![
                    ("ASDF_CONFIG_FILE".into(), asdf_config_file),
                    ("ASDF_DIR".into(), asdf_dir),
                    ("ASDF_DATA_DIR".into(), asdf_data_dir),
                    ("PATH".into(), path),
                ];
            }
        }
        Vec::new()
    }

    async fn install_version(
        &self,
        version: &str,
        dest: &Path,
        os: &str,
        _arch: &str,
    ) -> Result<()> {
        if os == "windows" {
            bail!("`asdf` can not be install on Windows")
        }

        let url = format!(
            "https://github.com/asdf-vm/asdf/archive/refs/tags/v{version}.tar.gz",
            version = version
        );
        let filename = format!("asdf-v{version}.tar.gz", version = version);
        let archive = self.download(&url, Some(filename), None).await?;

        self.extract(&archive, dest, 1)?;
        self.executables(dest, &["bin/asdf"])?;

        // TODO: use a setting to determine the keep downloads policy for both Stencila and asdf
        fs::write(dest.join(".asdfrc"), "always_keep_download = yes\n")?;

        Ok(())
    }
}

impl AsdfBinary {
    /// Add an `asdf` plugin
    pub async fn add_plugin(plugin: &str, repo: Option<&str>) -> Result<()> {
        let asdf = AsdfBinary {}.ensure().await?;

        let mut args = vec!["plugin", "add", plugin];
        if let Some(repo) = repo {
            args.push(repo);
        }
        asdf.run(&args).await?;

        Ok(())
    }

    /// List all versions for an `asdf` package
    pub async fn list_all(package: &str) -> Result<Vec<String>> {
        let asdf = AsdfBinary {}.ensure().await?;

        asdf.run_with(&["plugin", "add", package], None, None)
            .await
            .ok();

        let output = asdf
            .command()
            .args(&["list", "all", package])
            .output()
            .await?;
        let versions: Vec<String> = std::str::from_utf8(&output.stdout)?
            .lines()
            .map(String::from)
            .collect();

        Ok(versions)
    }

    /// Install a version for an `asdf` package
    ///
    /// Calls `uninstall` first because even with an empty directory it
    /// will consider it is already installed and do nothing.
    pub async fn install(package: &str, version: &str) -> Result<()> {
        let asdf = AsdfBinary {}.ensure().await?;

        asdf.run_with(&["plugin", "add", package], None, None)
            .await
            .ok();
        asdf.run_with(&["uninstall", package, version], None, None)
            .await
            .ok();
        asdf.run(&["install", package, version]).await?;

        Ok(())
    }

    /// Install a version for an `asdf` package with env vars
    pub async fn install_with(
        package: &str,
        version: &str,
        vars: &[(impl AsRef<OsStr>, impl AsRef<OsStr>)],
    ) -> Result<()> {
        let mut asdf = AsdfBinary {}.ensure().await?;

        asdf.env_list(vars);
        asdf.run(&["install", package, version]).await?;

        Ok(())
    }
}
