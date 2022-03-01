use async_trait::async_trait;
use defaults::Defaults;
use eyre::{bail, eyre, Result};
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
use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
};
use tokio::io::{AsyncBufReadExt, BufReader};

/// Re-exports for the convenience of crates implementing `BinaryTrait`
pub use ::async_trait;
pub use ::eyre;
pub use ::http_utils;
pub use ::serde_json;
pub use ::tokio;
pub use ::tracing;

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
    pub fn named(name: &str) -> Binary {
        Binary::new(name, &[], &[])
    }

    /// Get the directory where versions of a binary are installed by default
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

    /// Get the versions of the binary (synchronously)
    ///
    /// This is just the synchronous version of [`Self::versions`]
    fn versions_sync(&self, os: &str) -> Result<Vec<String>> {
        let os = os.to_string();
        let clone = self.clone_box();
        let (sender, receiver) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            let result = clone.versions(&os).await;
            sender.send(result)
        });
        receiver.recv()?
    }

    /// Update a list of versions with the result of a function (usually involving a remote request) which may fail
    ///
    /// This should be used in implementations of `versions` so that a re static list of versions can be augmented
    /// with the latest versions at runtime but not be dependent upon a network connection (or API rate limiting).
    fn versions_update_maybe(&self, versions: &[&str], more: Result<Vec<String>>) -> Vec<String> {
        let mut versions: Vec<String> = versions.iter().map(|str| str.to_string()).collect();

        if let Ok(more) = more {
            for version in more {
                if !versions.contains(&version) {
                    versions.push(version)
                }
            }
        };

        self.semver_versions_sorted(&versions)
    }

    /// Get the versions of the binary from GitHub REST API for repo releases
    ///
    /// This will usually be followed by a call to `semver_versions_sorted` or
    /// `semver_versions_matching`.
    ///
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

        let releases = http_utils::get_json(&format!(
            "https://api.github.com/repos/{}/{}/releases?per_page=100",
            org, repo
        ))
        .await?;

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

    /// Get the versions of the binary from GitHub REST API for repo tags
    ///
    /// Only includes tags that start with "v" and which are a valid version
    ///
    /// See https://docs.github.com/en/rest/reference/repos#list-repository-tags.
    #[cfg(feature = "download")]
    async fn versions_github_tags(&self, org: &str, repo: &str) -> Result<Vec<String>> {
        tracing::info!(
            "Getting list of tags for https://github.com/{}/{}",
            org,
            repo
        );

        let releases = http_utils::get_json(&format!(
            "https://api.github.com/repos/{}/{}/tags?per_page=100",
            org, repo
        ))
        .await?;

        let versions: Vec<String> = releases
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|release| {
                release["name"]
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
    fn install_env(&self, _version: Option<String>) -> Vec<(OsString, OsString)> {
        Vec::new()
    }

    /// Get the environment variables that should be set when the binary is run
    fn run_env(&self, _version: Option<String>) -> Vec<(OsString, OsString)> {
        Vec::new()
    }

    /// Parse a string as a semantic version
    fn semver_version(&self, string: &str) -> Result<semver::Version> {
        Ok(semver::Version::parse(string)?)
    }

    /// Parse a string as a semantic version and return the major version e.g. "3.10.2" => "3"
    fn semver_version_major(&self, string: &str) -> Result<String> {
        self.semver_version(string)
            .map(|version| version.major.to_string())
    }

    /// Parse a string as a semantic version and return the minor version e.g. "3.10.2" => "3.10"
    fn semver_version_minor(&self, string: &str) -> Result<String> {
        self.semver_version(string)
            .map(|version| format!("{}.{}", version.major, version.minor))
    }

    /// Parse a string as a semantic version requirement
    ///
    /// If the string is blank then semver `*` is assumed.
    ///
    /// If the string has no operator prefix (e.g. "="), that is, it is a version, not a requirement,
    /// then a semver operator is added as follows:
    ///
    /// - `x.y.z` => `=x.y.z`
    /// - `x.y` =>  `~x.y`
    /// - `x` =>  `^x`
    fn semver_requirement(&self, string: &str) -> Result<semver::VersionReq> {
        let string = string.strip_prefix('v').unwrap_or(string);
        let string = if string.trim().is_empty() {
            "*".to_string()
        } else {
            let op = string.chars().next().unwrap_or(' ');
            match "=^~><*".matches(op).count() {
                0 => match string.matches('.').count() {
                    2 => ["=", string].concat(),
                    1 => ["~", string].concat(),
                    _ => ["^", string].concat(),
                },
                _ => string.to_string(),
            }
        };

        match semver::VersionReq::parse(&string) {
            Ok(req) => Ok(req),
            Err(error) => {
                if error.to_string().contains("expected comma after") {
                    // Reattempt assuming user used a space instead of a comma (e.g. >=10.3 <17)
                    let joined = string.split(' ').collect::<Vec<_>>().join(",");
                    Ok(semver::VersionReq::parse(&joined)?)
                } else {
                    bail!(error)
                }
            }
        }
    }

    /// Does a version match a requirement?
    fn semver_version_matches(&self, version: &str, requirement: &str) -> Result<bool> {
        let requirement = self.semver_requirement(requirement)?;
        let version = self.semver_version(version)?;
        Ok(requirement.matches(&version))
    }

    // Filter out versions that are not valid semver versions and do not meet a semver requirement
    fn semver_versions_matching(&self, versions: &[String], requirement: &str) -> Vec<String> {
        let versions = self.semver_versions_sorted(versions);

        let requirement = match self.semver_requirement(requirement) {
            Ok(requirement) => requirement,
            Err(..) => return versions,
        };
        versions
            .iter()
            .filter_map(|version| match self.semver_version(version) {
                Ok(ver) => match requirement.matches(&ver) {
                    true => Some(version.to_string()),
                    false => None,
                },
                Err(..) => None,
            })
            .collect()
    }

    /// Filter out any versions that are not valid semver versions.
    /// Also sorts in **descending** semver order.
    fn semver_versions_sorted(&self, versions: &[String]) -> Vec<String> {
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

    /// Find the first installation of the binary on the `PATH`
    fn find(&self) -> Result<BinaryInstallation> {
        self.find_version_in(None, None)
    }

    /// Find the first installation of the binary on paths
    fn find_in(&self, paths: &OsStr) -> Result<BinaryInstallation> {
        self.find_version_in(None, Some(paths.into()))
    }

    /// Find the first installation of the binary, that matches semver requirement, on `PATH`
    fn find_version(&self, requirement: &str) -> Result<BinaryInstallation> {
        self.find_version_in(Some(requirement.into()), None)
    }

    /// Find the first installation of the binary, that matches semver requirement, on paths
    fn find_version_in(
        &self,
        requirement: Option<String>,
        paths: Option<OsString>,
    ) -> Result<BinaryInstallation> {
        let name = self.spec().name;
        let cwd = std::env::current_dir().unwrap();
        for path in which::which_in_all(name.clone(), paths, cwd)? {
            let version = self.version(&path);
            let env = self.run_env(version.clone());
            let installation = BinaryInstallation::new(name.clone(), path, version.clone(), env);
            if let Some(requirement) = &requirement {
                if let Some(version) = &version {
                    if self.semver_version_matches(version, requirement)? {
                        return Ok(installation);
                    }
                }
            } else {
                return Ok(installation);
            }
        }
        bail!(
            "No installation for binary `{}` matching semver requirement `{}` found",
            name,
            requirement.unwrap_or_else(|| "*".to_string())
        )
    }

    /// Ensure the binary is installed
    async fn ensure(&self) -> Result<BinaryInstallation> {
        self.ensure_version("*").await
    }

    /// Ensure a version of the binary that meets a semver requirement is installed
    async fn ensure_version(&self, requirement: &str) -> Result<BinaryInstallation> {
        let requirement = Some(requirement.to_string());
        match self.installed(requirement.clone())? {
            Some(installation) => Ok(installation),
            None => {
                self.install(requirement.clone()).await?;
                match self.installed(requirement.clone())? {
                    Some(installation) => Ok(installation),
                    None => bail!(
                        "Failed to ensure new installation of `{}`",
                        self.spec().name
                    ),
                }
            }
        }
    }

    /// Ensure a version of the binary that meets a semver requirement is installed (synchronously)
    ///
    /// This is just the synchronous version of [`Self::require`]
    fn ensure_version_sync(&self, requirement: &str) -> Result<BinaryInstallation> {
        let clone = self.clone_box();
        let requirement = requirement.to_string();
        let (sender, receiver) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            let result = clone.ensure_version(&requirement).await;
            sender.send(result)
        });
        receiver.recv()?
    }

    /// Require the binary
    async fn require(&self) -> Result<BinaryInstallation> {
        self.installed(None)?
            .ok_or_else(|| eyre!("No install of `{}` could be found", self.spec().name))
    }

    /// Require the binary (synchronously)
    ///
    /// This is just the synchronous version of [`Self::require`]
    fn require_sync(&self) -> Result<BinaryInstallation> {
        let clone = self.clone_box();
        let (sender, receiver) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            let result = clone.require().await;
            sender.send(result)
        });
        receiver.recv()?
    }

    /// Require a version of the binary that meets a semver requirement
    async fn require_version(&self, requirement: &str) -> Result<BinaryInstallation> {
        self.installed(Some(requirement.to_string()))?
            .ok_or_else(|| {
                eyre!(
                    "No version of `{}` meeting requirement `{}` could be found",
                    self.spec().name,
                    requirement
                )
            })
    }

    /// Require a version of the binary (synchronously)
    ///
    /// This is just the synchronous version of [`Self::require_version`]
    fn require_version_sync(&self, requirement: &str) -> Result<BinaryInstallation> {
        let clone = self.clone_box();
        let requirement = requirement.to_string();
        let (sender, receiver) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            let result = clone.require_version(&requirement).await;
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

        // Add the PATH var
        if let Some(path) = env::var_os("PATH") {
            tracing::trace!("Found $PATH: {:?}", path);
            let mut paths = env::split_paths(&path).collect();
            dirs.append(&mut paths);
        } else {
            tracing::trace!("No $PATH env var found");
        }

        // Join all the dirs together in a PATH style string to pass to `which_in_all`
        let dirs = match env::join_paths(dirs) {
            Ok(joined) => Some(joined),
            Err(error) => {
                tracing::warn!("While joining paths: {}", error);
                None
            }
        };

        // Search for executables with name or one of aliases
        let names = [vec![name.clone()], aliases].concat();
        let paths = names
            .iter()
            .map(|name| {
                match which::which_in_all(name, dirs.clone(), std::env::current_dir().unwrap()) {
                    Ok(paths) => paths.collect(),
                    Err(error) => {
                        tracing::warn!("While searching for binary {}: {}", name, error);
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
                let a = self.semver_version(a).unwrap();
                let b = self.semver_version(b).unwrap();
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
            let requirement = self.semver_requirement(&requirement)?;
            for install in installations {
                if let Some(version) = &install.version {
                    let version = self.semver_version(version)?;
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

    /// Get the highest installed version that matches a semver requirement (if specified)
    ///
    /// This is a convenience function that ignores errors in the semver requirement string
    /// and obtaining the version and will return `None` in both cases!
    fn installed_version(&self, requirement: Option<String>) -> Option<String> {
        self.installed(requirement)
            .ok()
            .flatten()
            .and_then(|installation| {
                installation
                    .version()
                    .ok()
                    .map(|version| version.to_string())
            })
    }

    /// Install the binary in the default location
    async fn install(&self, requirement: Option<String>) -> Result<String> {
        self.install_in(requirement, None).await
    }

    /// Install the binary in a specific directory
    async fn install_in(
        &self,
        requirement: Option<String>,
        dest: Option<PathBuf>,
    ) -> Result<String> {
        self.install_in_for(requirement, dest, None, None).await
    }

    /// Install the binary in a specific directory (synchronously)
    ///
    /// This is just the synchronous version of `Self::install_in`
    fn install_in_sync(
        &self,
        requirement: Option<String>,
        dest: Option<PathBuf>,
    ) -> Result<String> {
        let clone = self.clone_box();
        let (sender, receiver) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            let result = clone.install_in(requirement, dest).await;
            sender.send(result)
        });
        receiver.recv()?
    }

    /// Install the binary in a specific directory for a specific OS and architecture
    ///
    /// If a `requirement` is not supplied then the latest version is installed. If `requirement` is
    /// supplied then the latest version meeting its requirement is installed.
    async fn install_in_for(
        &self,
        requirement: Option<String>,
        dest: Option<PathBuf>,
        os: Option<String>,
        arch: Option<String>,
    ) -> Result<String> {
        let os = os.unwrap_or_else(|| OS.to_string());
        let arch = arch.unwrap_or_else(|| ARCH.to_string());
        let Binary { name, .. } = self.spec();

        let versions = self.versions(&os).await?;

        let requirement = requirement.unwrap_or_default().trim().to_string();
        let requirement = if requirement.is_empty() || requirement == "*" {
            match versions.first() {
                Some(version) => version.to_string(),
                None => bail!(
                    "Sorry, I don't know how to install `{}`; perhaps install it manually?",
                    name
                ),
            }
        } else {
            requirement
        };

        let requirement = self.semver_requirement(&requirement)?;

        // Get the latest version matching semver requirements
        if let Some(version) = versions.iter().find_map(|version| {
            match requirement.matches(
                &self
                    .semver_version(version)
                    .expect("Version to always be valid"),
            ) {
                true => Some(version),
                false => None,
            }
        }) {
            // Set install time env vars
            for (name, value) in self.install_env(Some(version.to_string())) {
                env::set_var(name, value)
            }

            // Ensure that the destination directory to exists
            let default_dest = self.dir(Some(version.to_string()), false)?;
            let dest = dest.unwrap_or(default_dest);
            fs::create_dir_all(&dest)?;
            let dest = dest.canonicalize()?;

            // Call the implementation
            self.install_version(version, &dest, &os, &arch).await?;

            tracing::info!("Installed `{} {}` to `{}`", name, version, dest.display());

            Ok(version.to_string())
        } else {
            bail!(
                "Sorry, I don't know how to install `{}` with version requirement `{}`. See `stencila binaries versions {}` or perhaps install it manually?",
                name,
                requirement,
                name
            )
        }
    }

    /// Install a specific version of the binary
    ///
    /// Implementations of this trait will usually override this method.
    async fn install_version(
        &self,
        _version: &str,
        _dest: &Path,
        _os: &str,
        _arch: &str,
    ) -> Result<()> {
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
        http_utils::download_file(url, &path).await?;

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

#[async_trait]
impl BinaryTrait for Binary {
    fn spec(&self) -> Binary {
        self.clone()
    }

    fn clone_box(&self) -> Box<dyn BinaryTrait> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BinaryInstallation {
    /// The name of the binary
    #[serde(skip)]
    pub name: String,

    /// The path the binary is installed to
    pub path: PathBuf,

    /// The version of the binary at the path
    pub version: Option<String>,

    /// The environment variables to set before the binary is executed
    #[serde(serialize_with = "serialize_env")]
    pub env: HashMap<OsString, OsString>,
}

/// Serialize the `env` property because `OSString` serialization is not human readable.
fn serialize_env<S>(env: &HashMap<OsString, OsString>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let env: HashMap<String, String> = env
        .iter()
        .map(|(key, value)| {
            (
                key.to_string_lossy().to_string(),
                value.to_string_lossy().to_string(),
            )
        })
        .collect();
    env.serialize(serializer)
}

impl BinaryInstallation {
    /// Create an instance
    pub fn new(
        name: impl AsRef<str>,
        path: impl AsRef<Path>,
        version: Option<String>,
        env: Vec<(OsString, OsString)>,
    ) -> BinaryInstallation {
        BinaryInstallation {
            name: name.as_ref().into(),
            path: path.as_ref().into(),
            version,
            env: env.into_iter().collect(),
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

    /// Get the version of this binary installation
    pub fn version(&self) -> Result<&str> {
        self.version.as_deref().ok_or_else(|| {
            eyre!(
                "Installation for `{}` at `{}` does not have a version",
                self.name,
                self.path.display()
            )
        })
    }

    /// Set the runtime environment for the binary using a map
    ///
    /// Sets the environment variables on the `Command` instance returned from
    /// `commmand` or `command_sync`.
    pub fn env_map(&mut self, vars: std::collections::hash_map::Iter<'_, OsString, OsString>) {
        self.env = vars.map(|(k, v)| (k.clone(), v.clone())).collect();
    }

    /// Set the runtime environment for the binary using a list of tuples
    ///
    /// Sets the environment variables on the `Command` instance returned from
    /// `commmand` or `command_sync`.
    pub fn env_list(&mut self, vars: &[(impl AsRef<OsStr>, impl AsRef<OsStr>)]) {
        self.env = vars
            .iter()
            .map(|var| (var.0.as_ref().to_owned(), var.1.as_ref().to_owned()))
            .collect();
    }

    /// Get the command for the binary
    pub fn command(&self) -> tokio::process::Command {
        let mut command = tokio::process::Command::new(&self.path);
        command.envs(&self.env);
        command
    }

    /// Get the synchronous command for the binary
    pub fn command_sync(&self) -> std::process::Command {
        let mut command = std::process::Command::new(&self.path);
        command.envs(&self.env);
        command
    }

    /// Run the binary, log any outputs on stdout and stderr, and fail if
    /// exit code is not 0.
    pub async fn run<I, S>(&self, args: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.run_with(args, Some(tracing::Level::INFO), Some(tracing::Level::INFO))
            .await
    }

    /// Run the binary, log any outputs on stdout and stderr, and fail if
    /// exit code is not 0.
    pub async fn run_with<I, S>(
        &self,
        args: I,
        stdout_log_level: Option<tracing::Level>,
        stderr_log_level: Option<tracing::Level>,
    ) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        tracing::trace!("Running binary installation {:?}", self);

        let mut child = self
            .command()
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(level) = stdout_log_level {
            let stdout = child.stdout.take().expect("stdout is piped");
            let mut stdout_reader = BufReader::new(stdout).lines();
            let name = self.name.clone();
            tokio::spawn(async move {
                while let Ok(Some(line)) = stdout_reader.next_line().await {
                    match level {
                        tracing::Level::ERROR => tracing::error!("[{}] {}", name, line),
                        tracing::Level::WARN => tracing::warn!("[{}] {}", name, line),
                        tracing::Level::INFO => tracing::info!("[{}] {}", name, line),
                        tracing::Level::DEBUG => tracing::info!("[{}] {}", name, line),
                        _ => tracing::trace!("[{}] {}", name, line),
                    }
                }
            });
        }

        if let Some(level) = stderr_log_level {
            let stderr = child.stderr.take().expect("stderr is piped");
            let mut stderr_reader = BufReader::new(stderr).lines();
            let name = self.name.clone();
            tokio::spawn(async move {
                while let Ok(Some(line)) = stderr_reader.next_line().await {
                    match level {
                        tracing::Level::ERROR => tracing::error!("[{}] {}", name, line),
                        tracing::Level::WARN => tracing::warn!("[{}] {}", name, line),
                        tracing::Level::INFO => tracing::info!("[{}] {}", name, line),
                        tracing::Level::DEBUG => tracing::debug!("[{}] {}", name, line),
                        _ => tracing::trace!("[{}] {}", name, line),
                    }
                }
            });
        }

        let exit_status = child.wait().await?;
        match exit_status.success() {
            true => Ok(()),
            false => bail!(
                "Binary `{}` ({}) exited with {}",
                self.name,
                self.path.display(),
                exit_status
            ),
        }
    }

    /// Run the binary synchronously, log any outputs on stdout and stderr, and fail if
    /// exit code is not 0.
    ///
    /// The sync version of `run`.
    pub fn run_sync<I, S>(&self, args: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let clone = self.clone();
        let args = args
            .into_iter()
            .map(|arg| arg.as_ref().to_owned())
            .collect::<Vec<_>>();
        let (sender, receiver) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            let result = clone.run(args).await;
            sender.send(result)
        });
        receiver.recv()?
    }

    /// Run the binary and connect to stdin, stdout and stderr streams
    pub fn interact(&self, args: &[&str]) -> Result<tokio::process::Child> {
        tracing::trace!("Interacting with binary installation {:?}", self);

        let result = self
            .command()
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        Ok(result?)
    }
}
