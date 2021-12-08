use async_trait::async_trait;
use defaults::Defaults;
use eyre::{bail, Result};
use regex::Regex;
use serde::Serialize;
#[allow(unused_imports)]
use std::{
    cmp::Ordering,
    env::{
        self,
        consts::{ARCH, OS},
    },
    fs, io,
    path::{Path, PathBuf},
    process::{Output, Stdio},
    str::FromStr,
};
use tokio::process::{Child, Command};

/// Re-exports for the convenience of crates implementing `BinaryTrait`
pub use ::async_trait;
pub use ::eyre;

/// Get the directory where binaries are stored
pub fn binaries_dir() -> PathBuf {
    let user_data_dir = dirs::data_dir().unwrap_or_else(|| env::current_dir().unwrap());
    match env::consts::OS {
        "macos" | "windows" => user_data_dir.join("Stencila").join("Binaries"),
        _ => user_data_dir.join("stencila").join("binaries"),
    }
}

/// A specification for a binary
///
/// Contains fields `name` and `aliases` for searching for existing installations
/// and `version_args` and `version` regex for determining the version of those.
///
/// The `installable` field lists the versions that Stencila is capable of installing.
#[derive(Defaults, Serialize)]
pub struct Binary {
    /// The name of the binary
    pub name: String,

    /// Any aliases used to search for the binary
    pub aliases: Vec<String>,

    /// Globs of paths that should be searched for the binary in addition
    /// to those on `$PATH`.
    ///
    /// On Windows (and potentially on other OSes) the installation directory
    /// may not necessarily be on the `$PATH`. This allows specifying additional
    /// directories that should be searched.
    pub globs: Vec<String>,

    /// The arguments used to get the version of the binary
    #[serde(skip)]
    #[def = r#"vec!["--version".to_string()]"#]
    pub version_args: Vec<String>,

    /// The regex used to get the version from the output of
    /// the binary.
    #[serde(skip)]
    #[def = r#"Regex::new("\\d+.\\d+(.\\d+)?").unwrap()"#]
    pub version_regex: Regex,

    /// Versions of the binary that can be installed by Stencila
    pub installable: Vec<String>,
}

impl Clone for Binary {
    fn clone(&self) -> Binary {
        Binary {
            name: self.name.clone(),
            aliases: self.aliases.clone(),
            globs: self.globs.clone(),
            installable: self.installable.clone(),
            ..Default::default()
        }
    }
}

impl Binary {
    /// Define a binary
    pub fn new(name: &str, aliases: &[&str], globs: &[&str], installable: &[&str]) -> Binary {
        Binary {
            name: name.to_string(),
            aliases: aliases
                .iter()
                .map(|s| String::from_str(s).unwrap())
                .collect(),
            globs: globs.iter().map(|s| String::from_str(s).unwrap()).collect(),
            installable: installable
                .iter()
                .map(|s| String::from_str(s).unwrap())
                .rev()
                .collect(),
            ..Default::default()
        }
    }

    /// Define an "unregistered" binary
    ///
    /// Used when we only know the name of the binary that the user is searching for
    /// and no nothing about aliases, path globs or how to install it.
    pub fn unregistered(name: &str) -> Binary {
        Binary::new(name, &[], &[], &[])
    }

    /// Get the directory where versions of a binary are installed
    fn dir(&self, version: Option<String>, ensure: bool) -> Result<PathBuf> {
        let dir = binaries_dir().join(&self.name);
        let dir = if let Some(version) = version {
            dir.join(version)
        } else {
            dir
        };

        if ensure {
            fs::create_dir_all(&dir)?
        }

        Ok(dir)
    }

    /// Get the version of the binary at a path
    ///
    /// Parses the output of the command and adds a `0` patch semver part if
    /// necessary.
    fn version(&self, path: &Path) -> Option<String> {
        let output = std::process::Command::new(path)
            .args(&self.version_args)
            .output();
        if let Ok(output) = output {
            for stream in [output.stdout, output.stderr] {
                let test = std::str::from_utf8(&stream).unwrap_or("");
                if let Some(version) = self.version_regex.captures(test).map(|captures| {
                    let mut parts: Vec<&str> = captures[0].split('.').collect();
                    while parts.len() < 3 {
                        parts.push("0")
                    }
                    parts.join(".")
                }) {
                    return Some(version);
                }
            }
        }
        None
    }

    /// Find installations of this binary
    fn installations(&self) -> Vec<BinaryInstallation> {
        let mut dirs: Vec<PathBuf> = Vec::new();

        // Collect the directories for previously installed versions
        if let Ok(dir) = self.dir(None, false) {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        // Search for binary in top level (Windows)
                        dirs.push(path.clone());
                        // Search for binary in `bin` (MacOS & Linux convention)
                        dirs.push(path.join("bin"))
                    }
                }
            }
        }
        tracing::debug!("Found Stencila install dirs: {:?}", dirs);

        // Collect the directories matching the globs
        if !self.globs.is_empty() {
            let mut globbed: Vec<PathBuf> = Vec::new();
            for pattern in &self.globs {
                let mut found = match glob::glob(pattern) {
                    Ok(found) => found.flatten().collect::<Vec<PathBuf>>(),
                    Err(..) => continue,
                };
                globbed.append(&mut found)
            }
            tracing::debug!("Found globbed dirs: {:?}", globbed);
            dirs.append(&mut globbed)
        }

        // Add the system PATH env var
        if let Some(path) = env::var_os("PATH") {
            tracing::debug!("Found $PATH: {:?}", path);
            let mut paths = env::split_paths(&path).collect();
            dirs.append(&mut paths);
        }

        // Join all the dirs together in a PATH style string to pass to `which_in_all`
        let dirs = if !dirs.is_empty() {
            match env::join_paths(dirs) {
                Ok(joined) => Some(joined),
                Err(error) => {
                    tracing::warn!("While joining paths: {}", error);
                    None
                }
            }
        } else {
            None
        };

        // Search for executables with name or one of aliases
        let names = [vec![self.name.clone()], self.aliases.clone()].concat();
        tracing::debug!("Searching for names: {:?}", names);
        let paths = names
            .iter()
            .map(|name| {
                match which::which_in_all(name, dirs.clone(), std::env::current_dir().unwrap()) {
                    Ok(paths) => paths.collect(),
                    Err(error) => {
                        tracing::warn!("While searching for executables for {}: {}", name, error);
                        Vec::new()
                    }
                }
            })
            .flatten();

        // Get version of each executable found
        // tracing::debug!("Getting versions for paths {:?}", paths);
        let mut installs: Vec<BinaryInstallation> = paths
            .map(|path| {
                BinaryInstallation::new(self.name.clone(), path.clone(), self.version(&path))
            })
            .collect();

        // Sort the installations by descending order of version so that
        // the most recent version (meeting semver requirements) is returned by `installation()`.
        installs.sort_by(|a, b| match (&a.version, &b.version) {
            (Some(a), Some(b)) => {
                let a = semver::Version::parse(a).unwrap();
                let b = semver::Version::parse(b).unwrap();
                a.partial_cmp(&b).unwrap_or(Ordering::Equal)
            }
            (Some(..), None) => Ordering::Greater,
            (None, Some(..)) => Ordering::Less,
            (None, None) => Ordering::Equal,
        });
        installs.reverse();

        installs
    }

    /// Are any versions installed that match the semver requirement (if specified)?
    fn installed(&self, semver: Option<String>) -> Result<Option<BinaryInstallation>> {
        let installations = self.installations();
        if let Some(semver) = semver {
            let semver = semver::VersionReq::parse(&semver)?;
            for install in installations {
                if let Some(version) = &install.version {
                    let version = semver::Version::parse(version)?;
                    if semver.matches(&version) {
                        return Ok(Some(install.clone()));
                    }
                }
            }
            Ok(None)
        } else if let Some(install) = installations.first() {
            Ok(Some(install.clone()))
        } else {
            Ok(None)
        }
    }
}

/// A trait for binaries
///
/// Allows specific binaries to override search, versioning and installation
/// methods. Usually only `install_version` should need to be overridden.
#[async_trait]
pub trait BinaryTrait: Send + Sync {
    /// Get the specification of the binary
    fn spec(&self) -> Binary;

    /// Get the directory where versions of a binary are installed
    fn dir(&self, version: Option<String>, ensure: bool) -> Result<PathBuf> {
        self.spec().dir(version, ensure)
    }

    /// Get the version of the binary at a path
    fn version(&self, path: &Path) -> Option<String> {
        self.spec().version(path)
    }

    /// Find installations of this binary
    fn installations(&self) -> Vec<BinaryInstallation> {
        self.spec().installations()
    }

    /// Are any versions installed that match the semver requirement (if specified)?
    fn installed(&self, semver: Option<String>) -> Result<Option<BinaryInstallation>> {
        self.spec().installed(semver)
    }

    /// Install the most recent version of the binary (meeting optional semver, OS, and arch requirements).
    async fn install(
        &self,
        semver: Option<String>,
        os: Option<String>,
        arch: Option<String>,
    ) -> Result<()> {
        let Binary {
            name, installable, ..
        } = self.spec();

        let semver = if let Some(semver) = semver {
            semver
        } else {
            installable
                .first()
                .expect("Always at least one version")
                .clone()
        };
        let semver = semver::VersionReq::parse(&semver)?;

        if let Some(version) = installable.iter().find_map(|version| {
            match semver
                .matches(&semver::Version::parse(version).expect("Version to always be valid"))
            {
                true => Some(version),
                false => None,
            }
        }) {
            let os = os.unwrap_or_else(|| OS.to_string());
            let arch = arch.unwrap_or_else(|| ARCH.to_string());
            self.install_version(version, &os, &arch).await?;
        } else {
            bail!(
                "Sorry, I don't know how to install `{}` version `{}`. See `stencila binaries installable` or perhaps install it manually?",
                name,
                semver
            )
        }

        Ok(())
    }

    /// Install a specific version of the binary
    async fn install_version(&self, _version: &str, _os: &str, _arch: &str) -> Result<()> {
        let spec = self.spec();
        bail!(
            "Installation of binary `{}` has not been implemented",
            spec.name
        )
    }

    /// Download a URL (usually an archive) to a temporary, but optionally cached, file
    #[cfg(feature = "reqwest")]
    async fn download(&self, url: &str) -> Result<PathBuf> {
        let url_parsed = url::Url::parse(url)?;
        let filename = url_parsed
            .path_segments()
            .expect("No segments in URL")
            .last()
            .expect("No file in URL");
        let path = binaries_dir().join("downloads").join(filename);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?
        }

        // Reuse downloaded files, only use this during development
        // and testing (to avoid multiple downloads) and avoid in production
        // in case a previous download was corrupted or similar.
        if path.exists() && cfg!(debug_assertions) {
            return Ok(path);
        }

        tracing::info!("📥 Downloading {} to {}", url, path.display());
        let response = reqwest::get(url).await?.error_for_status()?;
        let bytes = response.bytes().await?;
        let mut file = fs::File::create(&path)?;
        io::copy(&mut bytes.as_ref(), &mut file)?;

        Ok(path)
    }

    /// Extract an archive to a destination
    #[allow(unused_variables)]
    #[cfg(any(feature = "download-tar", feature = "download-zip"))]
    fn extract(&self, path: &Path, strip: usize, dest: &Path) -> Result<()> {
        tracing::info!("🔓 Extracting {} to {}", path.display(), dest.display());

        let ext = path
            .extension()
            .expect("Always has an extension")
            .to_str()
            .expect("Always convertible to str");

        match ext {
            #[cfg(feature = "download-zip")]
            "zip" => self.extract_zip(path, strip, dest),
            #[cfg(feature = "download-tar")]
            _ => self.extract_tar(ext, path, strip, dest),
            #[cfg(not(feature = "download-tar"))]
            _ => bail!("Downloading of archives has not been enabled"),
        }
    }

    /// Extract a tar archive
    #[cfg(feature = "download-tar")]
    fn extract_tar(&self, ext: &str, file: &Path, strip: usize, dest: &Path) -> Result<()> {
        let file = fs::File::open(&file)?;
        let mut archive = tar::Archive::new(match ext {
            "tar" => Box::new(file) as Box<dyn io::Read>,
            #[cfg(feature = "download-tar-gz")]
            "gz" | "tgz" => Box::new(flate2::read::GzDecoder::new(file)),
            #[cfg(feature = "download-tar-xz")]
            "xz" => Box::new(xz2::read::XzDecoder::new(file)),
            _ => bail!("Unhandled archive extension {}", ext),
        });

        let count = archive
            .entries()?
            .filter_map(|entry| entry.ok())
            .map(|mut entry| -> Result<()> {
                let mut path = entry.path()?.display().to_string();
                if strip > 0 {
                    let mut components: Vec<String> = path.split('/').map(String::from).collect();
                    components.drain(0..strip);
                    path = components.join("/")
                }

                let out_path = dest.join(&path);
                entry.unpack(&out_path).expect("Unable to unpack");

                Ok(())
            })
            .filter_map(|result| result.ok())
            .count();

        tracing::debug!("Extracted {} entries", count);

        Ok(())
    }

    /// Extract a zip archive
    #[cfg(feature = "download-zip")]
    fn extract_zip(&self, file: &Path, strip: usize, dest: &Path) -> Result<()> {
        let file = fs::File::open(&file)?;
        let mut archive = zip::ZipArchive::new(file)?;

        let mut count = 0;
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index)?;
            let mut path = entry
                .enclosed_name()
                .expect("Always has an enclosed name")
                .display()
                .to_string();

            if strip > 0 {
                let mut components: Vec<String> = path.split('/').map(String::from).collect();
                components.drain(0..strip);
                path = components.join("/")
            }

            let out_path = dest.join(&path);
            if let Some(parent) = out_path.parent() {
                if let Err(error) = fs::create_dir_all(parent) {
                    if error.kind() != io::ErrorKind::AlreadyExists {
                        bail!(error)
                    }
                }
            }

            if entry.is_file() {
                let mut out_file = fs::File::create(out_path)?;
                std::io::copy(&mut entry, &mut out_file)?;
                count += 1;
            }
        }

        tracing::debug!("Extracted {} entries", count);
        Ok(())
    }

    /// Make extracted files executable (if they exists)
    ///
    /// While tar archives retain permissions, zip archives do not,
    /// so we need to make sure to do this.
    fn executable(&self, dir: &Path, files: &[&str]) -> Result<()> {
        for file in files {
            let path = dir.join(file);
            if path.exists() {
                #[cfg(any(target_os = "linux", target_os = "macos"))]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
                }
            }
        }
        Ok(())
    }

    /// Uninstall a version, or all versions, of a binary
    async fn uninstall(&self, version: Option<String>) -> Result<()> {
        let dir = self.dir(version, false)?;
        if dir.exists() {
            fs::remove_dir_all(dir)?
        } else {
            tracing::warn!("No matching Stencila installed binary found")
        }

        Ok(())
    }
}

#[async_trait]
impl BinaryTrait for Binary {
    fn spec(&self) -> Binary {
        self.clone()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct BinaryInstallation {
    /// The name of the binary
    #[serde(skip)]
    pub name: String,

    /// The path the binary is installed to
    pub path: PathBuf,

    /// The version of the binary at the path
    pub version: Option<String>,
}

impl BinaryInstallation {
    /// Create an instance
    pub fn new(name: String, path: PathBuf, version: Option<String>) -> BinaryInstallation {
        BinaryInstallation {
            name,
            path,
            version,
        }
    }

    /// Get the command for the binary
    pub fn command(&self) -> Command {
        Command::new(&self.path)
    }

    /// Run the binary
    ///
    /// Returns the output of the command
    pub async fn run(&self, args: &[String]) -> Result<Output> {
        let output = self.command().args(args).output().await?;
        Ok(output)
    }

    /// Run the binary and connect to stdin, stdout and stderr streams
    ///
    /// Returns a `Child` process whose
    pub fn interact(&self, args: &[String]) -> Result<Child> {
        Ok(self
            .command()
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?)
    }
}
