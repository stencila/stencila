use async_trait::async_trait;
use defaults::Defaults;
use eyre::{bail, eyre, Result};
use once_cell::sync::Lazy;
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

/// Parse a string as a semantic version
pub fn semver_version(string: impl AsRef<str>) -> Result<semver::Version> {
    Ok(semver::Version::parse(string.as_ref())?)
}

/// Parse a string as a semantic version requirement
///
/// If the string has no operator prefix (e.g. "="), that is, it is a version, not a requirement,
/// then a semver operator is added as follows:
///
/// - `x.y.z` => `=x.y.z`
/// - `x.y` =>  `~x.y`
/// - `x` =>  `^x`
pub fn semver_requirement(string: impl AsRef<str>) -> Result<semver::VersionReq> {
    let string = string.as_ref();
    let op = string.chars().next().unwrap_or(' ');
    let string = match "=^~><*".matches(op).count() {
        0 => match string.matches('.').count() {
            2 => ["=", string].concat(),
            1 => ["~", string].concat(),
            _ => ["^", string].concat(),
        },
        _ => string.to_string(),
    };
    Ok(semver::VersionReq::parse(&string)?)
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
}

impl Clone for Binary {
    fn clone(&self) -> Binary {
        Binary {
            name: self.name.clone(),
            aliases: self.aliases.clone(),
            globs: self.globs.clone(),
            ..Default::default()
        }
    }
}

impl Binary {
    /// Define a binary
    pub fn new(name: &str, aliases: &[&str], globs: &[&str]) -> Binary {
        Binary {
            name: name.to_string(),
            aliases: aliases
                .iter()
                .map(|s| String::from_str(s).unwrap())
                .collect(),
            globs: globs.iter().map(|s| String::from_str(s).unwrap()).collect(),
            ..Default::default()
        }
    }

    /// Define an "unregistered" binary
    ///
    /// Used when we only know the name of the binary that the user is searching for
    /// and no nothing about aliases, path globs or how to install it.
    pub fn unregistered(name: &str) -> Binary {
        Binary::new(name, &[], &[])
    }

    /// Get the directory where versions of a binary are installed
    fn dir(&self, version: Option<String>, ensure: bool) -> Result<PathBuf> {
        let dir = binaries_dir().join("installs").join(&self.name);
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
    /// Parses the output of the command and adds a `0` patch part if necessary.
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
}

/// A trait for binaries
///
/// Allows specific binaries to override search, versioning and installation
/// methods. Usually only `install_version` should need to be overridden.
#[async_trait]
pub trait BinaryTrait: Send + Sync {
    /// Get the specification of the binary
    fn spec(&self) -> Binary;

    /// Create a `Box` of the trait
    ///
    /// This is needed because the `BinaryTrait` is not `Sized` and so can not
    /// be `Clone`. We need to be able to "clone" for methods such as `require_sync`.
    fn clone_box(&self) -> Box<dyn BinaryTrait>;

    /// Get the directory where versions of a binary are installed
    fn dir(&self, version: Option<String>, ensure: bool) -> Result<PathBuf> {
        self.spec().dir(version, ensure)
    }

    /// Get the versions of the binary that can be installed
    ///
    /// The returned list of string should be `major.minor.patch` semantic version
    /// numbers in **descending** order.
    ///
    /// This default implementation returns an empty list i.e. the binary is not
    /// installable by Stencila
    async fn versions(&self, _os: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    /// Get the versions of the binary from GitHub REST API
    ///
    /// This will usually be followed by a call to `semver_versions_sorted` or
    /// `semver_versions_matching`.
    ///
    /// Fetches the most recent thirty releases.
    /// At present this does not do authorization, so potentially runs foul of 60 req/s rate limiting.
    /// In the future, we may add authorization and/or caching to avoid hitting
    /// rate limit.
    ///
    /// See https://docs.github.com/en/rest/reference/releases.
    #[cfg(feature = "download")]
    async fn versions_github_releases(&self, org: &str, repo: &str) -> Result<Vec<String>> {
        tracing::info!(
            "Getting list of releases for https://github.com/{}/{}",
            org,
            repo
        );

        let url = format!(
            "https://api.github.com/repos/{}/{}/releases?per_page=100",
            org, repo
        );
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .header(
                "User-Agent",
                "Stencila (https://github.com/stencila/stencila/)",
            )
            .send()
            .await?
            .error_for_status()?;
        let releases: serde_json::Value = response.json().await?;

        let versions = releases
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|release| {
                release["tag_name"]
                    .as_str()
                    .map(|tag| tag.strip_prefix('v').unwrap_or(tag).to_string())
            })
            .collect();

        Ok(versions)
    }

    /// Get the version of the binary at a path
    fn version(&self, path: &Path) -> Option<String> {
        self.spec().version(path)
    }

    /// Get the environment variables that should be set when the binary is installed
    fn install_env(&self, _version: Option<String>) -> Vec<(String, String)> {
        Vec::new()
    }

    /// Get the environment variables that should be set when the binary is run
    fn run_env(&self, _version: Option<String>) -> Vec<(String, String)> {
        Vec::new()
    }

    /// Require a binary
    async fn require(&self, requirement: Option<String>, install: bool) -> Result<BinaryInstallation> {
        match self.installed(requirement.clone())? {
            Some(installation) => Ok(installation),
            None => {
                let spec = self.spec();
                if install {
                    self.install(requirement.clone(), None, None).await?;
                    match self.installed(requirement.clone())? {
                        Some(installation) => Ok(installation),
                        None => bail!("Failed to automatically install `{}`", spec.name),
                    }
                } else {
                    bail!(
                        "Binary `{}` with version requirement `{}` is not installed",
                        spec.name,
                        requirement.unwrap_or_default()
                    )
                }
            }
        }
    }

    /// Require a binary (synchronously)
    fn require_sync(&self, requirement: Option<String>, install: bool) -> Result<BinaryInstallation> {
        let clone = self.clone_box();
        let (sender, receiver) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            let result = clone.require(requirement, install).await;
            sender.send(result)
        });
        receiver.recv()?
    }

    /// Find installations of this binary
    fn installations(&self) -> Vec<BinaryInstallation> {
        let Binary {
            name,
            globs,
            aliases,
            ..
        } = self.spec();

        let mut dirs: Vec<PathBuf> = Vec::new();

        // Collect the directories of _all_ binaries installed by Stencila
        // We don't only include the folder for this particular binary because we want to be able to find
        // unregistered binaries such as `npx` and `RScript` that are installed alongside
        // binaries `node` and `R`.
        if let Ok(binary_dirs) = fs::read_dir(binaries_dir()) {
            for binary_dir in binary_dirs.flatten() {
                if let Ok(version_dirs) = fs::read_dir(binary_dir.path()) {
                    for version_dir in version_dirs.flatten() {
                        if let Ok(subdirs) = fs::read_dir(version_dir.path()) {
                            for dir in subdirs.flatten() {
                                let path = dir.path();
                                if path.is_dir() {
                                    // Search for binary in top level (Windows)
                                    dirs.push(path.clone());
                                    // Search for binary in `bin` (MacOS & Linux convention)
                                    dirs.push(path.join("bin"))
                                }
                            }
                        }
                    }
                }
            }
        }

        if !dirs.is_empty() {
            tracing::trace!("Found Stencila install dirs: {:?}", dirs);
        }

        // Collect the directories matching the globs
        if !globs.is_empty() {
            let mut globbed: Vec<PathBuf> = Vec::new();
            for pattern in globs {
                let mut found = match glob::glob(&pattern) {
                    Ok(found) => found.flatten().collect::<Vec<PathBuf>>(),
                    Err(..) => continue,
                };
                globbed.append(&mut found)
            }
            if !globbed.is_empty() {
                tracing::trace!("Found globbed dirs: {:?}", globbed);
            }
            dirs.append(&mut globbed)
        }

        // Add the system PATH
        // Cache the parsed PATH for efficiency
        static PATH: Lazy<Vec<PathBuf>> = Lazy::new(|| {
            if let Some(path) = env::var_os("PATH") {
                tracing::trace!("Found $PATH: {:?}", path);
                env::split_paths(&path).collect()
            } else {
                tracing::trace!("No $PATH env var found");
                Vec::new()
            }
        });
        dirs.append(&mut PATH.clone());

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
        let names = [vec![name.clone()], aliases].concat();
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
                let version = self.version(&path);
                let env = self.run_env(version.clone());
                BinaryInstallation::new(name.clone(), path, version, env)
            })
            .collect();

        // Sort the installations by descending order of version so that
        // the most recent version (meeting semver requirements) is returned by `installation()`.
        installs.sort_by(|a, b| match (&a.version, &b.version) {
            (Some(a), Some(b)) => {
                let a = semver_version(a).unwrap();
                let b = semver_version(b).unwrap();
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
    fn installed(&self, requirement: Option<String>) -> Result<Option<BinaryInstallation>> {
        tracing::debug!(
            "Checking if `{}` with requirement `{:?}` is installed",
            self.spec().name,
            requirement.clone().unwrap_or_else(|| "*".to_string())
        );

        let installations = self.installations();
        if let Some(requirement) = requirement {
            let requirement = semver_requirement(requirement)?;
            for install in installations {
                if let Some(version) = &install.version {
                    let version = semver_version(version)?;
                    if requirement.matches(&version) {
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

    /// Install the most recent version of the binary (meeting optional semver, OS, and arch requirements).
    ///
    /// If a `requirement` is not supplied then the latest version is installed. If `requirement` is
    /// supplied then the latest version meeting its requirement is installed.
    async fn install(
        &self,
        requirement: Option<String>,
        os: Option<String>,
        arch: Option<String>,
    ) -> Result<()> {
        let os = os.unwrap_or_else(|| OS.to_string());
        let arch = arch.unwrap_or_else(|| ARCH.to_string());
        let Binary { name, .. } = self.spec();

        let versions = self.versions(&os).await?;

        let requirement = if let Some(requirement) = requirement {
            requirement
        } else {
            match versions.first() {
                Some(version) => version.to_string(),
                None => bail!(
                    "Sorry, I don't know how to install `{}`; perhaps install it manually?",
                    name
                ),
            }
        };
        let requirement = semver_requirement(&requirement)?;

        // Get the latest version matching semver requirements
        if let Some(version) = versions.iter().find_map(|version| {
            match requirement.matches(&semver_version(version).expect("Version to always be valid"))
            {
                true => Some(version),
                false => None,
            }
        }) {
            for (name, value) in self.install_env(Some(version.to_string())) {
                env::set_var(name, value)
            }
            self.install_version(version, &os, &arch).await?;
        } else {
            bail!(
                "Sorry, I don't know how to install `{}` with requirement `{}`. See `stencila binaries versions {}` or perhaps install it manually?",
                name,
                requirement,
                name
            )
        }

        tracing::info!("Installed `{}`", name);

        Ok(())
    }

    /// Install a specific version of the binary
    ///
    /// Implementations of this trait will usually override this method.
    async fn install_version(&self, _version: &str, _os: &str, _arch: &str) -> Result<()> {
        let spec = self.spec();
        bail!(
            "Installation of binary `{}` has not been implemented",
            spec.name
        )
    }

    /// Download a URL (usually an archive) to a temporary, but optionally cached, file
    #[cfg(feature = "download")]
    async fn download(
        &self,
        url: &str,
        filename: Option<String>,
        directory: Option<PathBuf>,
    ) -> Result<PathBuf> {
        let filename = match filename {
            Some(filename) => filename,
            None => {
                let url_parsed = url::Url::parse(url)?;
                url_parsed
                    .path_segments()
                    .and_then(|segments| segments.last().map(|segment| segment.to_string()))
                    .ok_or_else(|| eyre!("Unable to determine filename"))?
            }
        };

        let directory =
            directory.unwrap_or_else(|| binaries_dir().join("downloads").join(self.spec().name));

        let path = directory.join(filename);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?
        }

        if path.exists() {
            return Ok(path);
        }

        tracing::info!("Downloading `{}` to `{}`", url, path.display());
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
        tracing::info!("Extracting `{}` to `{}`", path.display(), dest.display());

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
    /// # Arguments
    ///
    /// - `dir`: The directory that executable have been installed to
    ///
    /// - `paths`: The paths, within `dir`, that should be made executable;
    ///            can be Unix style forward slash paths and not all need to exist
    ///
    /// While tar archives retain permissions, zip archives do not,
    /// so we need to make sure to do this.
    fn executables(&self, dir: &Path, paths: &[&str]) -> Result<()> {
        for path in paths {
            let path = dir.join(path);
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
        let dir = self.dir(version.clone(), false)?;
        let name = self.spec().name;
        let version = version.unwrap_or_default();

        if dir.exists() {
            fs::remove_dir_all(dir)?;
            tracing::info!("Uninstalled `{}` {}", name, version);
        } else {
            tracing::warn!(
                "No Stencila-installed binary found for `{}` {}",
                name,
                version
            )
        }

        Ok(())
    }
}

/// A convenience macro for generating the required `clone_box` method
/// in types that implement `BinaryTrait`
#[macro_export]
macro_rules! binary_clone_box {
    () => {
        fn clone_box(&self) -> Box<dyn BinaryTrait> {
            Box::new(Self {})
        }
    };
}

/// Filter out any versions that are not valid semver versions.
/// Also sorts in descending **semver** order
pub fn semver_versions_sorted(versions: Vec<String>) -> Vec<String> {
    let mut versions: Vec<semver::Version> = versions
        .iter()
        .filter_map(|version| {
            // Parse a `VersionReq` rather than a `Version` because that allows for incomplete versions e.g. 2.15
            semver::VersionReq::parse(version)
                .ok()
                .and_then(|version_req| {
                    version_req.comparators.first().and_then(|comparator| {
                        // Ignore pre-releases
                        if comparator.pre.is_empty() {
                            Some(semver::Version::new(
                                comparator.major,
                                comparator.minor.unwrap_or(0),
                                comparator.patch.unwrap_or(0),
                            ))
                        } else {
                            None
                        }
                    })
                })
        })
        .collect();
    versions.dedup();
    versions.sort();
    versions.reverse();
    versions
        .iter()
        .map(|version| format!("{}.{}.{}", version.major, version.minor, version.patch))
        .collect()
}

// Filter out versions that are not valid semver versions and do not meet a semver requirement
pub fn semver_versions_matching(versions: Vec<String>, requirement: &str) -> Vec<String> {
    let versions = semver_versions_sorted(versions);

    let requirement = match semver_requirement(requirement) {
        Ok(requirement) => requirement,
        Err(..) => return versions,
    };
    versions
        .iter()
        .filter_map(|version| match semver_version(version) {
            Ok(ver) => match requirement.matches(&ver) {
                true => Some(version.to_string()),
                false => None,
            },
            Err(..) => None,
        })
        .collect()
}

#[async_trait]
impl BinaryTrait for Binary {
    fn spec(&self) -> Binary {
        self.clone()
    }

    fn clone_box(&self) -> Box<dyn BinaryTrait> {
        Box::new(self.clone())
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

    /// The environment variables to set before the binary is executed
    pub env: Vec<(String, String)>,
}

impl BinaryInstallation {
    /// Create an instance
    pub fn new(
        name: String,
        path: PathBuf,
        version: Option<String>,
        env_vars: Vec<(String, String)>,
    ) -> BinaryInstallation {
        BinaryInstallation {
            name,
            path,
            version,
            env: env_vars,
        }
    }

    // Get the parent of the install path
    pub fn parent(&self) -> Result<PathBuf> {
        self.path
            .parent()
            .map(|path| path.to_path_buf())
            .ok_or_else(|| {
                eyre!(
                    "Path for binary `{}` has no parent: {}",
                    self.name,
                    self.path.display()
                )
            })
    }

    // Get the grandparent of the install path
    pub fn grandparent(&self) -> Result<PathBuf> {
        self.parent()?
            .parent()
            .map(|path| path.to_path_buf())
            .ok_or_else(|| {
                eyre!(
                    "Path for binary `{}` has no parent: {}",
                    self.name,
                    self.path.display()
                )
            })
    }

    // Was this installed by Stencila?
    pub fn is_stencila_install(&self) -> bool {
        self.path.strip_prefix(binaries_dir()).is_ok()
    }

    pub fn version(&self) -> Result<&str> {
        self.version
            .as_deref()
            .ok_or_else(|| eyre!("Installation for `{}` does not have a version", self.name))
    }

    /// Get the command for the binary
    pub fn command(&self) -> tokio::process::Command {
        tokio::process::Command::new(&self.path)
    }

    /// Get the synchronous command for the binary
    pub fn command_sync(&self) -> std::process::Command {
        std::process::Command::new(&self.path)
    }

    /// Set the runtime environment for the binary
    pub fn set_env(&self) {
        for (name, value) in &self.env {
            env::set_var(name, value)
        }
    }

    /// Run the binary
    ///
    /// Returns the output of the command
    pub async fn run(&self, args: &[&str]) -> Result<Output> {
        tracing::trace!("Running binary installation {:?}", self);

        self.set_env();
        let output = self
            .command()
            .args(args)
            // TODO: instead of inheriting, forward to log INFO entries
            .stderr(Stdio::inherit())
            .output()
            .await?;
        Ok(output)
    }

    /// Run the binary synchronously
    ///
    /// The sync version of `run`. Returns the output of the command
    pub fn run_sync(&self, args: &[&str]) -> Result<Output> {
        tracing::trace!("Running binary installation {:?}", self);

        self.set_env();
        let output = self
            .command_sync()
            .args(args)
            // TODO: instead of inheriting, forward to log INFO entries
            .stderr(Stdio::inherit())
            .output()?;
        Ok(output)
    }

    /// Run the binary and connect to stdin, stdout and stderr streams
    ///
    /// Returns a `Child` process whose
    pub fn interact(&self, args: &[&str]) -> Result<tokio::process::Child> {
        tracing::trace!("Interacting with binary installation {:?}", self);

        self.set_env();
        Ok(self
            .command()
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?)
    }
}
