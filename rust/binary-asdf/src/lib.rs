use std::{env, fs};

use binary::{
    async_trait::async_trait,
    binaries_dir, binary_clone_box,
    eyre::{bail, Result},
    Binary, BinaryTrait,
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
    #[rustfmt::skip]
    fn spec(&self) -> Binary {
        Binary::new(
            "asdf",
            &[],
            &[],
            // Release list at https://github.com/asdf-vm/asdf/releases
            &[
                "0.9.0"
            ],
        )
    }

    binary_clone_box!();

    fn run_env(&self, version: Option<String>) -> Vec<(String, String)> {
        if let Ok(dir) = self.dir(version, false) {
            let binaries_dir = binaries_dir();
            if let Ok(..) = dir.strip_prefix(&binaries_dir) {
                let asdf_config_file = dir.join(".asdfrc").display().to_string();
                let asdf_dir = dir.display().to_string();
                let asdf_data_dir = binaries_dir.display().to_string();
                let path = format!("{}:{}", asdf_dir, env::var("PATH").unwrap_or_default());
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
