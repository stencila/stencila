use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub use eyre;
use eyre::{bail, Result};
pub use fs_utils;
pub use libcnb;
use libcnb::{
    data::build_plan::{BuildPlan as CnblibBuildPlan, BuildPlanBuilder},
    libcnb_runtime_build, libcnb_runtime_detect, BuildArgs, DetectArgs,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
pub use toml;
pub use tracing;

/// The stack id
///
/// Currently a constants (cause there's only one :) but in the
/// future there may be more e.g. `stencila.stacks.jammy`
const CNB_STACK_ID: &str = "stencila.stacks.focal";

/// Test whether the current CNB platform is Stencila
pub fn platform_is_stencila(platform_dir: &Path) -> bool {
    platform_dir.join("env").join("STENCILA_VERSION").exists()
}

/// A local trait for buildpacks that extends `libcnb::Buildpack`
///
/// Why? To provide some additional introspection and the ability
/// to compile several buildpacks into a single binary and call
/// their `detect` and `build` methods.
pub trait BuildpackTrait: libcnb::Buildpack {
    /// Get the content of the `buildpack.toml` file
    fn toml() -> &'static str;

    /// Get the buildpack's spec (a.k.a. descriptor) from the `buildpack.toml`
    fn spec() -> BuildpackToml {
        let toml = Self::toml();
        match toml::from_str::<BuildpackToml>(toml) {
            Ok(toml) => toml,
            Err(error) => {
                tracing::error!("While parsing buildpack.toml: {}", error);
                BuildpackToml::default()
            }
        }
    }

    /// Get the buildpack's id from its descriptor
    fn id() -> String {
        Self::spec().buildpack.id
    }

    /// Ensure the buildpack's runtime directory exists and return its path
    ///
    /// Both the `detect` and `build` methods require that `buildpack.toml`
    /// be available on disk.
    fn ensure_dir(&self) -> Result<String> {
        let dir = buildpacks_dir()?.join(Self::id());
        fs::create_dir_all(&dir)?;

        // Write the `buildpack.toml` to the directory
        let toml_path = dir.join("buildpack.toml");
        fs::write(&toml_path, Self::toml())?;

        Ok(dir.display().to_string())
    }

    /// Test whether any of the files exists in the working directory
    ///
    /// A convenience method for use in `detect`.
    fn any_exist(&self, paths: &[&str]) -> bool {
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
    fn file_contains(&self, file: &str, string: &str) -> bool {
        fs::read_to_string(file)
            .map(|content| content.contains(string))
            .unwrap_or(false)
    }

    /// Generate a `libcnb` build plan from a list of dependency names
    ///
    /// A convenience method for use in `detect` which ensures that each
    /// dependency is added as a `requires` and `provides`. If both of these
    /// are absent from the plan then `Pack` will fail `detect` for this buildpack.
    fn build_plan(&self, dependencies: &[&str]) -> CnblibBuildPlan {
        let mut builder = BuildPlanBuilder::new();
        for dependency in dependencies {
            builder = builder.requires(dependency).provides(dependency)
        }
        builder.build()
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
                Self::id(),
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
                Self::id(),
                error
            ),
        }
    }
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

// The `libcnb` crate provides similar structs to those below, often with stronger typing,
// but those do not implement `Serialize` or `Clone` and so for our purposes
// it was easier to reimplement them here.

/// A `struct` representing a `buildpack.toml` file
///
/// Used primarily to read in and display the spec for a buildpack for
/// use in commands such a `stencila buildpacks show`.
///
/// See https://github.com/buildpacks/spec/blob/main/buildpack.md#buildpacktoml-toml
#[skip_serializing_none]
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct BuildpackToml {
    pub api: String,
    pub buildpack: Buildpack,
    pub stacks: Option<Vec<BuildpackStack>>,
    pub order: Option<Vec<BuildpackGroup>>,
    pub metadata: Option<serde_json::Value>,
}

#[skip_serializing_none]
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Buildpack {
    pub id: String,
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

/// A `struct` representing a Build Plan (TOML)
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

/// A `struct` representing a Buildpack Plan (TOML)
///
/// See https://github.com/buildpacks/spec/blob/main/buildpack.md#buildpack-plan-toml
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct BuildpackPlan {
    pub entries: Vec<BuildPlanRequires>,
}
