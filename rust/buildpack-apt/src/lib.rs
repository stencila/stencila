use std::{
    collections::HashSet,
    env,
    ffi::OsString,
    fs::{create_dir_all, read_dir, read_to_string, remove_file, write},
    path::{Path, PathBuf},
    process,
};

use binary::{http_utils::download_sync, Binary, BinaryTrait};
use buildpack::{
    eyre::{self, eyre},
    libcnb::{
        self,
        build::{BuildContext, BuildResult, BuildResultBuilder},
        data::{build_plan::BuildPlan, layer_content_metadata::LayerTypes, layer_name},
        detect::{DetectContext, DetectResult, DetectResultBuilder},
        generic::{GenericMetadata, GenericPlatform},
        layer::{ExistingLayerStrategy, Layer, LayerResult, LayerResultBuilder},
        layer_env::{LayerEnv, ModificationBehavior, Scope},
        Buildpack,
    },
    maplit::hashmap,
    tracing, BuildpackTrait, LayerOptions,
};
use serde::{Deserialize, Serialize};
use utils::vec_string;

pub struct AptBuildpack;

impl BuildpackTrait for AptBuildpack {
    fn toml() -> &'static str {
        include_str!("../buildpack.toml")
    }
}

// The name of the file that is detected in the app dir
const APT_FILE: &str = "Aptfile";

// The name of the layer that the buildpack creates
const APT_PACKAGES: &str = "apt_packages";

impl Buildpack for AptBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = eyre::Report;

    fn detect(&self, _context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        // Detect `Aptfile`
        let aptfile = PathBuf::from(APT_FILE);

        // Get the Linux release for reuse below
        let linux_flavour = sys_info::linux_os_release().ok();

        // Fail if no Aptfile, or Aptfile exists but not on Ubuntu Linux
        if !aptfile.exists() {
            return DetectResultBuilder::fail().build();
        } else if env::consts::OS != "linux"
            || linux_flavour
                .as_ref()
                .map_or_else(|| "".to_string(), |rel| rel.id().to_string())
                != "ubuntu"
        {
            tracing::warn!("Aptfile detected but will be ignored because not on Ubuntu Linux");
            return DetectResultBuilder::fail().build();
        }

        let mut build_plan = BuildPlan::new();

        // Require `apt_packages` layer if there is an `Aptfile`
        if aptfile.exists() {
            let version = linux_flavour
                .expect("Should have returned by now if not on Linux")
                .version_codename
                .expect("Should have an Ubuntu version codename");

            let (require, provide) = Self::require_and_provide(
                APT_PACKAGES,
                APT_FILE,
                format!("Install `apt` packages for Ubuntu '{}'", version).trim(),
                Some(hashmap! {
                    "version" => version,
                    "file" => APT_FILE.to_string()
                }),
            );
            build_plan.requires.push(require);
            build_plan.provides.push(provide);
        }

        DetectResultBuilder::pass().build_plan(build_plan).build()
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        let entries = self.buildpack_plan_entries(&context.buildpack_plan);

        if let Some(options) = entries.get(APT_PACKAGES) {
            context.handle_layer(
                layer_name!("apt_packages"),
                AptPackagesLayer::new(options, Some(&context.app_dir)),
            )?;
        }

        BuildResultBuilder::new().build()
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AptPackagesLayer {
    /// The version of Ubuntu that packages will be installed for e.g `bionic`, `focal`
    version: String,

    /// The path to the `Aptfile` (or similar name) that specifies packages to be installed
    file: Option<PathBuf>,

    /// Should Ubuntu deb repository mirrors be used?
    mirrors: bool,

    /// Should packages that are no longer in the Aptfile be removed
    clean: bool,

    /// A list of package names, or deb URLs to be installed
    ///
    /// Usually instead of an `Aptfile` but can be specified in addition to it
    packages: Vec<String>,

    /// A list of repos to be used
    ///
    /// Usually instead of `:repo:` entries in an `Aptfile` but can be specified in addition to it
    repos: Vec<String>,
}

impl AptPackagesLayer {
    pub fn new(options: &LayerOptions, app_path: Option<&Path>) -> Self {
        let version = match options.get("version") {
            Some(version) => version.to_string(),
            None => sys_info::linux_os_release()
                .ok()
                .and_then(|info| info.version_codename)
                .unwrap_or_default(),
        };

        let file = options.get("file").map(PathBuf::from);

        // Split `Aptfile` into  packages and repos and detect options
        let mut mirrors = env::var("STENCILA_APT_MIRRORS").ok();
        let mut clean = env::var("STENCILA_APT_CLEAN").ok();
        let mut repos = Vec::new();
        let mut packages = match (&file, &app_path) {
            (Some(file), Some(path)) => read_to_string(path.join(file))
                .unwrap_or_default()
                .lines()
                .filter_map(|line| {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        None
                    } else if let Some(repo) = line.strip_prefix(":repo:") {
                        repos.push(repo.to_string());
                        None
                    } else if let Some(value) = line.strip_prefix(":mirrors:") {
                        mirrors = Some(value.trim().to_string());
                        None
                    } else if let Some(value) = line.strip_prefix(":clean:") {
                        clean = Some(value.trim().to_string());
                        None
                    } else {
                        Some(line.to_string())
                    }
                })
                .collect(),
            _ => Vec::new(),
        };

        // Turn off use of mirrors?
        let mirrors = !matches!(
            mirrors.as_deref(),
            Some("no") | Some("off") | Some("false") | Some("0")
        );

        // Turn off cleaning?
        let clean = !matches!(
            clean.as_deref(),
            Some("no") | Some("off") | Some("false") | Some("0")
        );

        // Add any other packages
        if let Some(list) = options.get("packages") {
            packages.append(&mut list.split(',').map(|pkg| pkg.trim().to_string()).collect());
        }

        // Add any other repos
        if let Some(list) = options.get("repos") {
            repos.append(&mut list.split(',').map(|pkg| pkg.trim().to_string()).collect());
        }

        Self {
            version,
            file,
            mirrors,
            clean,
            packages,
            repos,
        }
    }
}

impl Layer for AptPackagesLayer {
    type Buildpack = AptBuildpack;
    type Metadata = AptPackagesLayer;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            build: true,
            launch: true,
            cache: true,
        }
    }

    fn existing_layer_strategy(
        &self,
        _context: &BuildContext<Self::Buildpack>,
        layer_data: &libcnb::layer::LayerData<Self::Metadata>,
    ) -> Result<libcnb::layer::ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        let existing = &layer_data.content_metadata.metadata;
        let strategy = if self.version != existing.version {
            tracing::info!(
                "Existing `apt_packages` layer is for different Ubuntu version (`{}` => `{}`); will recreate",
                existing.version,
                self.version,
            );
            ExistingLayerStrategy::Recreate
        } else if self.repos != existing.repos {
            tracing::info!(
                "Existing `apt_packages` layer has different repos (`{}` => `{}`); will recreate",
                existing.repos.join(","),
                self.repos.join(","),
            );
            ExistingLayerStrategy::Recreate
        } else if self.packages != existing.packages {
            tracing::info!(
                "Existing `apt_packages` layer has different packages (`{}` => `{}`); will update",
                existing.packages.join(","),
                self.packages.join(",")
            );
            ExistingLayerStrategy::Update
        } else {
            tracing::info!("Existing `apt_packages` layer meets requirements; will keep",);
            ExistingLayerStrategy::Keep
        };
        Ok(strategy)
    }

    fn create(
        &self,
        _context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, eyre::Report> {
        tracing::info!("Creating `apt_packages` layer");
        self.install(layer_path)
    }

    fn update(
        &self,
        _context: &BuildContext<Self::Buildpack>,
        layer_data: &libcnb::layer::LayerData<Self::Metadata>,
    ) -> Result<LayerResult<Self::Metadata>, <Self::Buildpack as Buildpack>::Error> {
        tracing::info!("Updating `apt_packages` layer");
        self.install(&layer_data.path)
    }
}

impl AptPackagesLayer {
    pub fn install(
        &self,
        layer_path: &Path,
    ) -> Result<LayerResult<AptPackagesLayer>, eyre::Report> {
        let layer_path = &layer_path.canonicalize()?;

        // Create the directories that `apt-get` needs
        let apt_cache_dir = layer_path.join("cache");
        let apt_archives_dir = apt_cache_dir.join("archives");
        let apt_state_dir = layer_path.join("state");
        let apt_sources_dir = layer_path.join("sources");
        create_dir_all(apt_archives_dir.join("partial"))?;
        create_dir_all(apt_state_dir.join("lists").join("partial"))?;
        create_dir_all(&apt_sources_dir)?;

        // Create a list of base deb repositories
        let repos = if self.mirrors {
            // Generate a new sources list using the mirror protocol
            // In the future we may allow the `STENCILA_APT_MIRRORS` env var to contain a
            // list of mirrors to use
            let release = sys_info::linux_os_release()
                .ok()
                .and_then(|info| info.version_codename)
                .ok_or_else(|| eyre!("Unable to get Linux OS release"))?;
            format!(
                r#"
deb mirror://mirrors.ubuntu.com/mirrors.txt {release} main restricted universe multiverse
deb mirror://mirrors.ubuntu.com/mirrors.txt {release}-updates main restricted universe multiverse
deb mirror://mirrors.ubuntu.com/mirrors.txt {release}-backports main restricted universe multiverse
deb mirror://mirrors.ubuntu.com/mirrors.txt {release}-security main restricted universe multiverse
            "#,
            )
        } else {
            // Use the existing system sources list
            read_to_string(
                PathBuf::from("/")
                    .join("etc")
                    .join("apt")
                    .join("sources.list"),
            )?
        };

        // Add any repositories added in the `Aptfile`
        let repos = [&repos, "\n", &self.repos.join("\n")].concat();

        let apt_sources_list = apt_sources_dir.join("sources.list");
        write(&apt_sources_list, repos)?;

        // Configure apt-get and update cache
        let apt = Binary::named("apt-get").require_sync()?;
        let apt_options: Vec<String> = vec![
            "debug::nolocking=true",
            &format!("dir::cache={}", apt_cache_dir.display()),
            &format!("dir::state={}", apt_state_dir.display()),
            &format!("dir::etc::sourcelist={}", apt_sources_list.display()),
            "dir::etc::sourceparts=/dev/null",
        ]
        .into_iter()
        .map(|option| ["-o", option].concat())
        .collect();

        tracing::info!("Updating apt caches");
        apt.run_sync([apt_options.clone(), vec_string!["update"]].concat())?;

        // Read in the list of packages that are currently installed
        let installed_packages_dir = layer_path.join("installed").join("packages");
        create_dir_all(&installed_packages_dir)?;
        let mut installed_packages = read_dir(&installed_packages_dir)?
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect::<HashSet<String>>();

        // Ensure the `installed/debs` dir is created (reading of this done later only if needed)
        let installed_debs_dir = layer_path.join("installed").join("debs");
        create_dir_all(&installed_debs_dir)?;

        let dpkg = Binary::named("dpkg").require_sync()?;

        // Closure to get a list of the debs in archives dir
        let get_debs = || -> Vec<OsString> {
            apt_archives_dir
                .read_dir()
                .expect("Archives directory should be readable")
                .flatten()
                .filter_map(|entry| {
                    let path = entry.path();
                    if path.extension() == Some(&OsString::from("deb")) {
                        path.file_name().map(|name| name.to_os_string())
                    } else {
                        None
                    }
                })
                .collect()
        };

        // Get deb files, including those of dependencies, extract them and record the list
        // of files associated with each
        for package in &self.packages {
            // Slugify URLs to be more filesystem friendly
            let package_id = if package.starts_with("http") && package.ends_with(".deb") {
                package
                    .replace("://", "-")
                    .replace("/", "-")
            } else {
                package.to_string()
            };

            // If the package has already been installed then skip it (but remove so it is not
            // uninstalled later since it is still wanted)
            if installed_packages.remove(&package_id) {
                tracing::info!("Package `{}` is already installed", package);
                continue;
            } else {
                tracing::info!("Installing package `{}`", package);
            }

            // Record list of debs in archive before download
            let debs_before = get_debs();

            // Download debs for this package (including any dependencies if not a URL)
            if package.starts_with("http") && package.ends_with(".deb") {
                tracing::info!("Downloading `{}`", package);

                let path = apt_archives_dir.join(format!("{}.deb", package_id));
                download_sync(package, &path)?;
            } else {
                tracing::info!("Fetching deb files for package `{}`", package);

                // Assumes using `apt-get` >= 1.1 which replaced `--force-yes` with `--allow-*` options
                apt.run_sync(
                    [
                        apt_options.clone(),
                        vec_string![
                            "--assume-yes",
                            "--allow-downgrades",
                            "--allow-remove-essential",
                            "--allow-change-held-packages",
                            "--download-only",
                            "--reinstall",
                            "install",
                            package
                        ],
                    ]
                    .concat(),
                )?;
            }

            // Record the debs that were downloaded for the package
            // TODO: This is not very reliable since it will be empty if the package has
            // already been downloaded because it is an dependency of another.
            let debs_after = get_debs();
            let debs_downloaded: Vec<OsString> = debs_after
                .into_iter()
                .filter(|item| !debs_before.contains(item))
                .collect();

            // Extract the downloaded deb files into the layer and record
            tracing::info!("Extracting debs for package `{}`", package);
            for deb in &debs_downloaded {
                let deb_path = apt_archives_dir.join(deb);
                dpkg.run_sync([
                    "--extract",
                    &deb_path.display().to_string(),
                    &layer_path.display().to_string(),
                ])?;

                // Now that the deb has been extracted write it's manifest file with the list of files extracted
                // TODO: This does not need to be done here but can instead deferred to if / when the
                // package needs to be removed.
                let contents = process::Command::new("dpkg")
                    .arg("--contents")
                    .arg(deb_path)
                    .output()?
                    .stdout;
                let files_extracted = String::from_utf8(contents)?
                    .split('\n')
                    .filter_map(|line| {
                        let mut cols = line.split_whitespace();
                        let size = cols.nth(2).unwrap_or("0");
                        if size != "0" {
                            line.rfind("./").map(|pos| line[pos..].to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("\n");
                write(installed_debs_dir.join(deb), files_extracted)?;
            }

            // Now that the package has been successfully installed write its manifest file
            write(
                installed_packages_dir.join(package_id),
                debs_downloaded
                    .iter()
                    .map(|deb| deb.to_string_lossy().to_string())
                    .collect::<Vec<String>>()
                    .join("\n"),
            )?;
        }

        // Function to read a manifests file (list of debs, or files within a deb)
        fn read_manifest(file: &Path) -> Option<Vec<String>> {
            read_to_string(file).ok().map(|content| {
                content
                    .split('\n')
                    .map(|line| line.to_string())
                    .collect::<Vec<String>>()
            })
        }

        // Remove previously installed but currently unwanted packages (those not yet removed from the list)
        if self.clean && !installed_packages.is_empty() {
            for package in installed_packages {
                tracing::info!("Uninstalling package `{}`", package);

                // Read in the list of debs installed for this package
                let package_debs = installed_packages_dir.join(&package);
                let debs = read_manifest(&package_debs).unwrap_or_default();

                if debs.len() > 1 {
                    tracing::warn!("Dependencies were installed when package `{}` was installed. These will be removed also but may this may affect other packages subsequently installed that share those dependencies", package);
                }

                for deb in debs {
                    // Read in the list of files that were installed for the deb and remove them all
                    let deb_files = installed_debs_dir.join(&deb);
                    if let Some(files) = read_manifest(&deb_files) {
                        for file_path in files {
                            let layer_file_path = layer_path.join(file_path);
                            remove_file(layer_file_path).ok();
                        }
                    }

                    // Remove the deb from the archive
                    // If we don't do this then if the package get's re-added we do not "see"
                    // the deb as getting added. Also, it saves space.
                    remove_file(apt_archives_dir.join(&deb)).ok();

                    // Remove the manifest
                    remove_file(&deb_files).ok();
                }

                remove_file(&package_debs).ok();
            }
        }

        // Prepend a lot of env vars

        let prefix_paths = |paths: &[&str]| {
            // The trailing colon here is important to separate what we prepend
            // from the existing path
            env::join_paths(paths.iter().map(|path| layer_path.join(path)))
                .map(|joined| format!("{}:", joined.to_string_lossy()))
        };

        let mut layer_env = LayerEnv::new().chainable_insert(
            Scope::All,
            ModificationBehavior::Prepend,
            "PATH",
            prefix_paths(&["usr/bin", "usr/local/bin"])?,
        );

        let include_path_prepend = prefix_paths(&[
            "usr/include/x86_64-linux-gnu",
            "usr/include/i386-linux-gnu",
            "usr/include",
        ])?;
        for var in ["INCLUDE_PATH", "CPATH", "CPPPATH"] {
            layer_env.insert(
                Scope::All,
                ModificationBehavior::Prepend,
                var,
                &include_path_prepend,
            );
        }

        let library_paths = prefix_paths(&[
            "usr/lib/x86_64-linux-gnu",
            "usr/lib/i386-linux-gnu",
            "usr/lib",
            "lib/x86_64-linux-gnu",
            "lib/i386-linux-gnu",
            "lib",
        ])?;
        for var in ["LD_LIBRARY_PATH", "LIBRARY_PATH"] {
            layer_env.insert(
                Scope::All,
                ModificationBehavior::Prepend,
                var,
                &library_paths,
            );
        }

        layer_env.insert(
            Scope::All,
            ModificationBehavior::Prepend,
            "PKG_CONFIG_PATH",
            prefix_paths(&[
                "usr/lib/x86_64-linux-gnu/pkgconfig",
                "usr/lib/i386-linux-gnu/pkgconfig",
                "usr/lib/pkgconfig",
            ])?,
        );

        LayerResultBuilder::new(self.clone()).env(layer_env).build()
    }
}
