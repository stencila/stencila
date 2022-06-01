use std::{
    collections::HashMap,
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

pub use eyre;
use eyre::{bail, Result};
pub use fs_utils;
pub use hash_utils;
pub use libcnb;
use libcnb::{
    data::{
        build_plan::{Provide, Require},
        buildpack::BuildpackId,
    },
    layer_env::{LayerEnv, Scope},
    libcnb_runtime_build, libcnb_runtime_detect, BuildArgs, DetectArgs, Env,
};
pub use maplit;
pub use serde;
use serde::{Deserialize, Serialize};
pub use serde_json;
use serde_with::skip_serializing_none;
pub use tokio;
pub use toml;
pub use tracing;

/// The stack id
///
/// Currently a constant (cause there's only one :) but in the
/// future there may be more e.g. `stencila.stacks.jammy`
const CNB_STACK_ID: &str = "stencila.stacks.focal";

/// Test whether the current CNB platform is Stencila
pub fn platform_dir_is_stencila(platform_dir: &Path) -> bool {
    platform_dir.join("env").join("STENCILA_VERSION").exists()
}

pub trait BuildpackContext {
    /// Is this a local build
    ///
    /// For local builds, buildpacks use optimizations such as sym-linking to
    /// binaries on the host machine. In contrast, for traditional CNB builds
    /// (e.g. using Pack) and for Stencila builds inside containers there
    /// should be no reliance on the host system (everything should be inside
    /// the `/layers` directory).
    ///
    /// Note we can't just use "are we inside a container" detection methods
    /// since when running in a microVM these won't work. Also, using presence
    /// of `/layers` is useful as it allows for testing during development.
    fn is_local(&self) -> bool {
        !(env::consts::OS == "linux" && PathBuf::from("/layers").exists())
    }
}

impl<B: libcnb::Buildpack> BuildpackContext for libcnb::detect::DetectContext<B> {}

impl<B: libcnb::Buildpack> BuildpackContext for libcnb::build::BuildContext<B> {}

/// A local trait for buildpacks that extends `libcnb::Buildpack`
///
/// Why? To provide some additional introspection and the ability
/// to compile several buildpacks into a single binary and call
/// their `detect` and `build` methods.
pub trait BuildpackTrait: libcnb::Buildpack {
    /// Get the content of the `buildpack.toml` file
    fn toml() -> &'static str;

    /// Get the buildpack's spec (a.k.a. descriptor) from the `buildpack.toml`
    fn spec() -> Result<BuildpackToml> {
        let toml = Self::toml();
        let spec = toml::from_str::<BuildpackToml>(toml)?;
        Ok(spec)
    }

    /// Get the buildpack's `id` from the `buildpack.toml`
    fn id(&self) -> Result<BuildpackId> {
        let spec = Self::spec()?;
        Ok(spec.buildpack.id)
    }

    /// Ensure the buildpack's runtime directory exists and return its path
    ///
    /// Both the `detect` and `build` methods require that `buildpack.toml`
    /// be available on disk.
    fn ensure_dir(&self) -> Result<String> {
        let id = self.id()?;
        let dir = buildpacks_dir()?.join(id.to_string());
        fs::create_dir_all(&dir)?;

        // Write the `buildpack.toml` to the directory
        let toml_path = dir.join("buildpack.toml");
        fs::write(&toml_path, Self::toml())?;

        Ok(dir.display().to_string())
    }

    /// Test whether any of the files exists in the working directory
    ///
    /// A convenience method for use in `detect`.
    fn any_exist(paths: &[&str]) -> bool {
        for path in paths {
            if PathBuf::from(path).exists() {
                return true;
            }
        }
        false
    }

    /// Test whether a file contains a string
    ///
    /// A convenience method for use in `detect`.
    fn file_contains(file: &str, string: &str) -> bool {
        fs::read_to_string(file)
            .map(|content| content.contains(string))
            .unwrap_or(false)
    }

    /// Prepend a path to the `PATH` env var
    fn prepend_path(first: &Path) -> Result<OsString> {
        if let Some(path) = env::var_os("PATH") {
            let mut paths = vec![first.into()];
            paths.append(&mut env::split_paths(&path).collect::<Vec<_>>());
            Ok(env::join_paths(paths)?)
        } else {
            Ok(first.into())
        }
    }

    /// Parses a `.tool-versions` file (if any)
    ///
    /// A convenience method for use in `detect`. If there is no such file, then
    /// the returned map will be empty.
    ///
    /// Note that each line of `.tool-versions` can have multiple versions, but
    /// that this takes the first.
    fn tool_versions() -> HashMap<String, String> {
        match fs::read_to_string(".tool-versions") {
            Ok(content) => content
                .lines()
                .filter_map(|line| {
                    let mut parts = line.split_whitespace();
                    let name = match parts.next() {
                        Some(name) => name.to_string(),
                        None => return None,
                    };
                    let version = parts.next().unwrap_or("*").to_string();
                    Some((name, version))
                })
                .collect(),
            Err(..) => HashMap::new(),
        }
    }

    /// Generate `Require` with source, description, and options
    ///
    /// Used in `detect` to specify which layers the buildpack requires
    fn require(
        name: impl AsRef<str>,
        source: impl AsRef<str>,
        desc: impl AsRef<str>,
        options: Option<HashMap<&str, String>>,
    ) -> Require {
        let mut require = Require::new(name.as_ref());
        if !source.as_ref().is_empty() {
            require.metadata.insert(
                "source".to_string(),
                toml::Value::String(source.as_ref().to_string()),
            );
        }
        if !desc.as_ref().is_empty() {
            require.metadata.insert(
                "description".to_string(),
                toml::Value::String(desc.as_ref().to_string()),
            );
        }
        if let Some(options) = options {
            for (key, value) in options {
                require
                    .metadata
                    .insert(key.to_string(), toml::Value::String(value));
            }
        }

        require
    }

    /// Generate `Require` and `Provide` objects
    ///
    /// Used in `detect` to specify which layers the buildpack must build.
    fn require_and_provide(
        name: impl AsRef<str>,
        source: impl AsRef<str>,
        desc: impl AsRef<str>,
        options: Option<HashMap<&str, String>>,
    ) -> (Require, Provide) {
        (
            Self::require(&name, source, desc, options),
            Provide::new(name.as_ref()),
        )
    }

    /// Get all the environment variables of the process
    ///
    /// Used at the start of `build` to save the current state of the environment
    /// (so it can be restored later).
    fn get_env_vars(&self) -> HashMap<OsString, OsString> {
        env::vars_os().collect()
    }

    /// Set the environment variables defined in a buildpack layer
    ///
    /// This simulates what the CNB platform does during the build phase
    /// between buildpacks, but does it between layers. This is useful because some layers might
    /// be dependent upon things installed in the previous layer. By setting
    /// env vars like `PATH` we avoid all sorts on shenanigans in the subsequent layers
    /// for referring to the previous ones (e.g. `node_modules` layer needing `node` installed
    /// in `node` layer).
    ///
    /// Currently only sets the layer env vars that have [`Scope::Build`] and does
    /// not remove existing env vars.
    fn set_layer_env_vars(&self, layer_env: &LayerEnv) {
        let env = layer_env.apply(Scope::Build, &Env::from_current());
        for (key, value) in env.iter() {
            env::set_var(key, value)
        }
    }

    /// Restore environment variables of the process
    ///
    /// Used at the end of `build` to restore environment variables previously
    /// saved using `get_env_vars`.
    ///
    /// In many use cases, restoring the env vars will not be necessary
    /// (e.g. when running an individual buildpack as part of a CNB build) but to avoid potential
    /// conflicts is encouraged as part of the `build()` method.
    fn restore_env_vars(&self, vars: HashMap<OsString, OsString>) {
        for (key, ..) in env::vars_os() {
            env::remove_var(key);
        }
        for (key, value) in vars {
            env::set_var(key, value)
        }
    }

    /// Parse the `entries` of a buildpack plan into layer name and associated metadata
    ///
    /// A convenience method for use in `build`.
    fn buildpack_plan_entries(
        &self,
        buildpack_plan: &libcnb::data::buildpack_plan::BuildpackPlan,
    ) -> HashMap<String, LayerOptions> {
        buildpack_plan
            .entries
            .iter()
            .map(|entry| {
                let name = entry.name.clone();
                let options: HashMap<String, String> = entry
                    .metadata
                    .iter()
                    .map(|(key, value)| {
                        (key.clone(), value.as_str().unwrap_or_default().to_string())
                    })
                    .collect();
                (name, options)
            })
            .collect()
    }

    /// Run the buildpack's `detect` method
    fn detect_with(&self, platform_dir: &Path, build_plan: &Path) -> Result<i32>
    where
        Self: Sized,
    {
        env::set_var("CNB_STACK_ID", CNB_STACK_ID);

        let buildpack_dir = self.ensure_dir()?;
        env::set_var("CNB_BUILDPACK_DIR", buildpack_dir);

        match libcnb_runtime_detect(
            self,
            DetectArgs {
                platform_dir_path: PathBuf::from(platform_dir),
                build_plan_path: PathBuf::from(build_plan),
            },
        ) {
            Ok(code) => Ok(code),
            Err(error) => bail!(
                "While running `detect` for buildpack `{}`: {}",
                self.id()?,
                error
            ),
        }
    }

    /// Run the buildpack's `build` method
    fn build_with(
        &self,
        layers_dir: &Path,
        platform_dir: &Path,
        buildpack_plan: &Path,
    ) -> Result<i32>
    where
        Self: Sized,
    {
        env::set_var("CNB_STACK_ID", CNB_STACK_ID);

        let buildpack_dir = self.ensure_dir()?;
        env::set_var("CNB_BUILDPACK_DIR", buildpack_dir);

        match libcnb_runtime_build(
            self,
            BuildArgs {
                layers_dir_path: PathBuf::from(layers_dir),
                platform_dir_path: PathBuf::from(platform_dir),
                buildpack_plan_path: PathBuf::from(buildpack_plan),
            },
        ) {
            Ok(code) => Ok(code),
            Err(error) => bail!(
                "While running `build` for buildpack `{}`: {}",
                self.id()?,
                error
            ),
        }
    }
}

/// Type for `requires.metadata`
pub type LayerOptions = HashMap<String, String>;

/// Metadata for a layer that uses a hash to determine existing layer strategy
#[derive(Clone, Deserialize, Serialize)]
pub struct LayerHashMetadata {
    pub hash: String,
}

/// Metadata for a layer that uses a version to determine existing layer strategy
#[derive(Clone, Deserialize, Serialize)]
pub struct LayerVersionMetadata {
    pub version: String,
}

/// Get the path of the directory where buildpacks stored on the users machine
pub fn buildpacks_dir() -> Result<PathBuf> {
    let user_data_dir = dirs::data_dir()
        .unwrap_or_else(|| env::current_dir().expect("Should always be able to get current dir"));
    let dir = match env::consts::OS {
        "macos" | "windows" => user_data_dir.join("Stencila").join("Buildpacks"),
        _ => user_data_dir.join("stencila").join("buildpacks"),
    };
    Ok(dir)
}

/// Generate a locally unique tag for an image based on a directory path
///
/// Build a container image with a tag `<name>-<hash>` where `<name>` is the
/// name of the directory containing the `Dockerfile` and `<hash>` is the 12-character
/// truncated SHA256 hash of its path (to avoid clashes between directories with the same name).
pub fn tag_for_path(path: &Path) -> String {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed".to_string());

    let mut hash = hash_utils::str_sha256_hex(&path.display().to_string());
    hash.truncate(12);

    [&name, "-", &hash].concat()
}

// The `libcnb` crate provides similar structs to those below, often with stronger typing,
// but those do not implement `Serialize` or `Clone` and so for our purposes
// it was easier to reimplement them here. This decision should be revisited at some time.

/// A Buildpack Descriptor (`buildpack.toml`)
///
/// Used primarily to read in and display the spec for a buildpack for
/// use in commands such a `stencila buildpacks show <label>`.
///
/// See https://github.com/buildpacks/spec/blob/main/buildpack.md#buildpacktoml-toml
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BuildpackToml {
    pub api: String,
    pub buildpack: Buildpack,
    pub stacks: Option<Vec<BuildpackStack>>,
    pub order: Option<Vec<BuildpackGroup>>,
    pub metadata: Option<serde_json::Value>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Buildpack {
    pub id: BuildpackId,
    pub version: String,
    pub name: String,
    pub clear_env: Option<bool>,
    pub homepage: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<Vec<String>>,
    #[serde(rename = "sbom-formats")]
    pub sbom_formats: Option<Vec<String>>,
    pub licenses: Option<Vec<BuildpackLicense>>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BuildpackLicense {
    pub r#type: Option<String>,
    pub uri: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BuildpackStack {
    pub id: String,
    pub mixins: Option<Vec<String>>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct BuildpackGroup {
    pub id: String,
    pub version: String,
    pub optional: bool,
}

/// A Build Plan
///
/// Generated by the `detect` method of a buildpack (if it matches against a folder).
///
/// See https://github.com/buildpacks/spec/blob/main/buildpack.md#build-plan-toml
#[skip_serializing_none]
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct BuildPlan {
    pub provides: Option<Vec<BuildPlanProvides>>,
    pub requires: Option<Vec<BuildPlanRequires>>,
    pub or: Option<Vec<BuildPlanOr>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BuildPlanProvides {
    pub name: String,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct BuildPlanRequires {
    pub name: String,
    pub metadata: Option<toml::Value>,
}

#[skip_serializing_none]
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct BuildPlanOr {
    pub provides: Option<BuildPlanProvides>,
    pub requires: Option<BuildPlanRequires>,
}

/// A Buildpack Plan
///
/// Passed to the `build` method of each buildpack involved in the build.
///
/// See https://github.com/buildpacks/spec/blob/main/buildpack.md#buildpack-plan-toml
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct BuildpackPlan {
    pub entries: Vec<BuildPlanRequires>,
}

/// Runtime entry point function
///
/// Just sets up logging and calls `libcnb::libcnb_runtime`
/// to do the actual work.
pub fn runtime<B: libcnb::Buildpack>(buildpack: &B) {
    tracing_subscriber::fmt().init();
    libcnb::libcnb_runtime(buildpack);
}

/// Generate a `main` function for a buildpack
#[macro_export]
macro_rules! buildpack_main {
    ($buildpack:ident) => {
        #[tokio::main]
        async fn main() {
            buildpack::runtime(&$buildpack);
        }
    };
}
