use std::{
    env::consts::{ARCH, OS},
    fs::{File, create_dir_all},
    io::Cursor,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, SystemTime},
};

use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use zip::ZipArchive;

use cli_utils::color_print::cstr;
use common::{
    clap::{self, Parser},
    eyre::{Report, Result, bail},
    once_cell::sync::Lazy,
    reqwest::{Client, header::USER_AGENT},
    serde_json,
    tar::Archive,
    tempfile,
    tokio::{
        self,
        fs::{self, read_to_string, write},
        task::JoinHandle,
    },
    tracing,
};
use dirs::{DirType, get_app_dir};
use version::{STENCILA_USER_AGENT, STENCILA_VERSION};

/// Upgrade the Stencila CLI to the latest version
pub async fn upgrade(force: bool) -> Result<Option<String>> {
    let latest = GithubRelease::latest().await?;

    if !force && latest.version() == *STENCILA_VERSION {
        return Ok(None);
    }

    let temp = tempfile::tempdir()?;
    let path = latest.download(temp.path()).await?;

    tracing::debug!("Replacing binary with `{}`", path.display());
    self_replace::self_replace(path)?;

    Ok(Some(latest.version()))
}

static UPGRADE_AVAILABLE: Lazy<AtomicBool> = Lazy::new(AtomicBool::default);

/// Check if an upgrade is available
///
/// This spawns a background task so as to not block the main task
/// of the CLI. A check is only done if one has not been done recently,
/// unless `force = true`.
pub fn check(force: bool) -> JoinHandle<Option<String>> {
    let check = async move {
        let cache = get_app_dir(DirType::Cache, true)?.join("latest-release.json");

        let fetch = if !force && cache.exists() {
            let metadata = fs::metadata(&cache).await?;
            let modification_time = metadata.modified()?;
            SystemTime::now().duration_since(modification_time)? > Duration::from_secs(3600 * 24)
        } else {
            true
        };

        let latest = if fetch {
            let latest = GithubRelease::latest().await?;

            let json = serde_json::to_string(&latest)?;
            write(cache, json).await?;

            latest
        } else {
            let json = read_to_string(&cache).await?;
            serde_json::from_str(&json)?
        };

        // Both versions are expected to be semver, but if they are not then, then
        // the will return None
        if let (Ok(latest_semver), Ok(current_semver)) = (
            semver::Version::parse(&latest.version()),
            semver::Version::parse(STENCILA_VERSION),
        ) {
            if latest_semver > current_semver {
                UPGRADE_AVAILABLE.store(true, Ordering::SeqCst);
                Ok::<_, Report>(Some(latest.version()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    };
    tokio::spawn(async {
        match check.await {
            Ok(version) => version,
            Err(error) => {
                tracing::debug!("While checking for upgrade: {error}");
                None
            }
        }
    })
}

/// Notify the user if a upgrade is available on stderr
#[allow(clippy::print_stderr)]
pub fn notify() {
    if UPGRADE_AVAILABLE.load(Ordering::SeqCst) {
        eprintln!("🎂 A newer version is available. Get it using `stencila upgrade`");
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct GithubRelease {
    tag_name: String,
}

impl GithubRelease {
    /// Get the latest release
    async fn latest() -> Result<GithubRelease> {
        tracing::debug!("Getting latest release");

        let client = Client::new();
        let response = client
            .get("https://api.github.com/repos/stencila/stencila/releases/latest")
            .header(USER_AGENT, STENCILA_USER_AGENT)
            .send()
            .await?;

        Ok(response.json().await?)
    }

    /// Extract the version from the tag name
    fn version(&self) -> String {
        self.tag_name
            .strip_prefix('v')
            .unwrap_or(&self.tag_name)
            .to_string()
    }

    /// Download and extract the binary for current platform
    async fn download(&self, dir: &Path) -> Result<PathBuf> {
        let tag_name = &self.tag_name;
        let folder_name = format!(
            "cli-{}-{}-{}",
            tag_name,
            match ARCH {
                "x86_64" => "x86_64",
                "aarch64" => "aarch64",
                _ => bail!("Unsupported architecture"),
            },
            match OS {
                "linux" => "unknown-linux-gnu",
                "macos" => "apple-darwin",
                "windows" => "pc-windows-msvc",
                _ => bail!("Unsupported OS"),
            }
        );
        let file_name = format!(
            "{}{}",
            folder_name,
            match OS {
                "linux" => ".tar.gz",
                "macos" => ".tar.gz",
                "windows" => ".zip",
                _ => bail!("Unsupported OS"),
            }
        );

        let url = format!(
            "https://github.com/stencila/stencila/releases/download/{tag_name}/{file_name}"
        );

        tracing::debug!("Downloading latest release from {url}");
        let client = Client::new();
        let response = client.get(url).send().await?.bytes().await?;

        tracing::debug!("Extracting latest release");
        match OS {
            "linux" | "macos" => {
                let tar_gz = Cursor::new(response);
                let tar = GzDecoder::new(tar_gz);
                let mut archive = Archive::new(tar);
                archive.unpack(dir)?;
            }
            "windows" => {
                let cursor = Cursor::new(response);
                let mut archive = ZipArchive::new(cursor)?;
                for i in 0..archive.len() {
                    let mut file = archive.by_index(i)?;
                    let out_path = match file.enclosed_name() {
                        Some(path) => dir.join(path),
                        None => continue,
                    };

                    if (*file.name()).ends_with('/') {
                        create_dir_all(&out_path)?;
                    } else {
                        if let Some(parent) = out_path.parent()
                            && !parent.exists()
                        {
                            create_dir_all(parent)?;
                        }
                        let mut out_file = File::create(&out_path)?;
                        std::io::copy(&mut file, &mut out_file)?;
                    }
                }
            }
            _ => bail!("Unsupported OS"),
        };

        let path = dir.join(folder_name).join(match OS {
            "windows" => "stencila.exe",
            _ => "stencila",
        });

        Ok(path)
    }
}

/// Upgrade to the latest version
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// Perform upgrade even if the current version is the latest
    #[arg(long, short)]
    force: bool,

    /// Check for an available upgrade but do not install it
    #[arg(long, short)]
    check: bool,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Upgrade to the latest version</dim>
  <b>stencila upgrade</b>

  <dim># Check if an upgrade is available without installing</dim>
  <b>stencila upgrade</b> <c>--check</c>

  <dim># Force upgrade even if current version is latest</dim>
  <b>stencila upgrade</b> <c>--force</c>

<bold><b>Note</b></bold>
  Upgrade downloads the latest release from GitHub and replaces
  the current binary.
"
);

impl Cli {
    #[allow(clippy::print_stderr)]
    pub async fn run(self) -> Result<()> {
        if self.check {
            match check(true).await? {
                Some(version) => {
                    eprintln!("🎂 Upgrade available: {STENCILA_VERSION} → {version}");
                }
                None => {
                    eprintln!(
                        "👍 No upgrade needed: current version {STENCILA_VERSION} is the latest"
                    );
                }
            }
        } else if let Some(version) = upgrade(self.force).await? {
            eprintln!("🍰 Successfully upgraded to version {version}");
        } else {
            eprintln!(
                "🙌 Current version {STENCILA_VERSION} is the latest (use --force to override)"
            );
        }

        Ok(())
    }
}
