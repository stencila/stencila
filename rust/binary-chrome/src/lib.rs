use binary::{
    async_trait::async_trait,
    eyre::{bail, Result},
    Binary, BinaryTrait,
};

pub struct ChromeBinary {}

#[async_trait]
impl BinaryTrait for ChromeBinary {
    fn spec(&self) -> Binary {
        Binary::new(
            "chrome",
            &["chromium"],
            // Version history at https://en.wikipedia.org/wiki/Google_Chrome_version_history
            // but only use triples ending in `.0` here and make sure there is a mapping in the
            // `install_version` method.
            &["91.0.0"],
        )
    }

    async fn install_version(&self, version: &str, os: &str, _arch: &str) -> Result<()> {
        // Chrome uses a peculiar version system with the build number
        // at the third position and not every build for every OS. So, use minor version
        // for mapping
        let minor_version = version.split('.').take(2).collect::<Vec<&str>>().join(".");

        // Map the minor_version to a "position" number which can be obtained from
        // https://vikyd.github.io/download-chromium-history-version
        let suffix = match minor_version.as_ref() {
            "91.0" => match os {
                "macos" => "Mac/869727/chrome-mac.zip",
                "windows" => "Win_x64/867878/chrome-win.zip",
                "linux" => "Linux_x64/860960/chrome-linux.zip",
                _ => bail!("Unmapped OS '{}'", os),
            },
            _ => bail!("Unmapped version number '{}'", version),
        };

        let url = format!(
            "https://www.googleapis.com/download/storage/v1/b/chromium-browser-snapshots/o/{suffix}?alt=media",
            suffix = suffix.replace("/", "%2F")
        );

        let archive = self.download(&url).await?;
        let dest = self.dir(Some(version.into()), true)?;
        self.extract(&archive, 1, &self.dir(Some(version.into()), true)?)?;
        self.executable(&dest, &["chrome", "chrome.exe"])?;

        Ok(())
    }
}
