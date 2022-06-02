use std::{
    env::{current_dir, set_current_dir},
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use once_cell::sync::Lazy;
use regex::Regex;

use binary_pack::{BinaryTrait, PackBinary};
use buildpack::{
    buildpacks_dir,
    eyre::{bail, Context, Result},
    libcnb::data::{buildpack::BuildpackId, buildpack_id},
    platform_dir_is_stencila, tag_for_path, toml, tracing, BuildPlan, BuildpackPlan, BuildpackToml,
    BuildpackTrait,
};

/// The set of registered buildpacks in the current process
pub static PACKS: Lazy<Arc<Buildpacks>> = Lazy::new(|| Arc::new(Buildpacks::new()));

/// A set of registered buildpacks
///
/// At present all buildpacks are builtin, but, as for `codecs`, `parsers` and other
/// collections it is possible that plugins could also provide buildpacks.
pub struct Buildpacks {
    inner: Vec<BuildpackToml>,
}

/// A macro to dispatch methods to builtin buildpacks
macro_rules! dispatch_builtins {
    ($label:expr, $method:ident $(,$arg:expr)*) => {
        match $label {
            #[cfg(feature = "buildpack-apt")]
            "stencila/apt" => buildpack_apt::AptBuildpack{}.$method($($arg),*),
            #[cfg(feature = "buildpack-dockerfile")]
            "stencila/dockerfile" => buildpack_dockerfile::DockerfileBuildpack{}.$method($($arg),*),
            #[cfg(feature = "buildpack-node")]
            "stencila/node" => buildpack_node::NodeBuildpack{}.$method($($arg),*),
            #[cfg(feature = "buildpack-python")]
            "stencila/python" => buildpack_python::PythonBuildpack{}.$method($($arg),*),
            #[cfg(feature = "buildpack-r")]
            "stencila/r" => buildpack_r::RBuildpack{}.$method($($arg),*),
            #[cfg(feature = "buildpack-sources")]
            "stencila/sources" => buildpack_sources::SourcesBuildpack{}.$method($($arg),*),
            #[cfg(feature = "buildpack-stencila")]
            "stencila/stencila" => buildpack_stencila::StencilaBuildpack{}.$method($($arg),*),
            _ => bail!("No buildpack with label `{}`", $label)
        }
    };
}

impl Buildpacks {
    /// Create a new buildpack registry
    pub fn new() -> Self {
        let specs: Vec<Option<BuildpackToml>> = vec![
            // The `SourcesBuildpack` should be first because it will import files
            // during `detect` which other buildpacks will run their `detect` against
            #[cfg(feature = "buildpack-sources")]
            buildpack_sources::SourcesBuildpack::spec().ok(),
            // The `Dockerfile` buildpack should come second because it excludes the
            // other buildpacks from running
            #[cfg(feature = "buildpack-dockerfile")]
            buildpack_dockerfile::DockerfileBuildpack::spec().ok(),
            #[cfg(feature = "buildpack-apt")]
            buildpack_apt::AptBuildpack::spec().ok(),
            #[cfg(feature = "buildpack-node")]
            buildpack_node::NodeBuildpack::spec().ok(),
            #[cfg(feature = "buildpack-python")]
            buildpack_python::PythonBuildpack::spec().ok(),
            #[cfg(feature = "buildpack-r")]
            buildpack_r::RBuildpack::spec().ok(),
            // Stencila CLI buildpack
            #[cfg(feature = "buildpack-stencila")]
            buildpack_stencila::StencilaBuildpack::spec().ok(),
        ];
        Self {
            inner: specs.into_iter().flatten().collect(),
        }
    }

    /// List the available buildpacks
    pub fn list(&self) -> &Vec<BuildpackToml> {
        &self.inner
    }

    /// Generate a Markdown table of the list of available buildpacks
    pub fn list_as_markdown(list: &[BuildpackToml]) -> String {
        let cols = "|--|----|-------|--------|";
        let head = "|Id|Name|Version|Keywords|";
        let body = list
            .iter()
            .map(|buildpack_toml| {
                let buildpack = &buildpack_toml.buildpack;
                format!(
                    "|{}|{}|{}|{}|",
                    buildpack.id,
                    buildpack.name,
                    buildpack.version,
                    buildpack
                        .keywords
                        .as_ref()
                        .map_or_else(String::new, |keywords| keywords.join(", "))
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
        format!(
            "{top}\n{head}\n{align}\n{body}\n{bottom}\n",
            top = cols,
            head = head,
            align = cols,
            body = body,
            bottom = cols
        )
    }

    /// Find the `id` of the first buildbpack matching the label
    pub fn find(&self, label: &str) -> Result<&BuildpackId> {
        let label_lower = label.to_ascii_lowercase();
        for buildpack_toml in &self.inner {
            let id = buildpack_toml.buildpack.id.to_string();
            if id == label_lower || id.split('/').last() == Some(label_lower.as_str()) {
                return Ok(&buildpack_toml.buildpack.id);
            }
        }
        bail!("No buildpack with id or label `{}`", label)
    }

    /// Get the buildpack with the given label or id
    pub fn get(&self, label: &str) -> Result<&BuildpackToml> {
        let buildpack_id = self.find(label)?;
        for buildpack_toml in &self.inner {
            if buildpack_toml.buildpack.id == *buildpack_id {
                return Ok(buildpack_toml);
            }
        }
        bail!("No buildpack with id or label `{}`", label)
    }

    /// Get the path to the CNB platform directory
    ///
    /// This directory path is passed to buildpacks in the `detect` and `build` phases.
    /// As a CNB platform, Stencila has its own platform directory which contains a
    /// file named `STENCILA_VERSION` containing the current version. This allows build packs
    /// to act differently depending if the platform is Stencila (local installs) versus
    /// Pack (install into Docker images).
    fn platform_dir() -> Result<PathBuf> {
        let dir = buildpacks_dir()?.join("platform");

        let env = dir.join("env");
        fs::create_dir_all(&env)?;

        let stencila_version = env.join("STENCILA_VERSION");
        fs::write(&stencila_version, env!("CARGO_PKG_VERSION"))?;

        Ok(dir)
    }

    /// Slugify a `BuildpackId` for use within a path
    ///
    /// Converts non-alphanumeric characters to underscores as seems
    /// to be the convention used by Pack
    fn slugify_buildpack_id(buildpack_id: &BuildpackId) -> String {
        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new("[^A-Za-z0-9 ]").expect("Unable to create regex"));

        REGEX.replace(buildpack_id.as_str(), "_").to_string()
    }

    /// Create a CNB layers directory for a buildpack
    ///
    /// Used in `build` when the `layers_dir` argument is not supplied (usually this only
    /// happens when called from within Stencila and not by a CNB platform tool such as `pack`).
    ///
    /// If there is a directory `/layers` on the filesystem, that will be used (usually when
    /// running inside a container), otherwise `./.stencila/layers` within the working directory
    /// will be used.
    fn layers_dir_default(working_dir: &Path, buildpack_id: &BuildpackId) -> Result<PathBuf> {
        let root_layers = PathBuf::from("/layers");
        let all_layers = if root_layers.exists() {
            root_layers
        } else {
            working_dir.join(".stencila").join("layers")
        };

        let buildpack_layers = all_layers.join(Self::slugify_buildpack_id(buildpack_id));
        
        fs::create_dir_all(&buildpack_layers).wrap_err(format!(
            "Could not create layers directory `{}` for buildpack `{}`",
            buildpack_layers.display(),
            buildpack_id.as_str()
        ))?;

        Ok(buildpack_layers)
    }

    /// Create a `.stencila/build/<buildpack>` directory in a working directory
    ///
    /// Rather than generate a temporary directory (as Pack does) we generate
    /// it within the `.stencila` directory for transparency and easier debugging and
    /// to be able to display the build plan to users.
    fn build_dir_default(working_dir: &Path, buildpack_id: &BuildpackId) -> Result<PathBuf> {
        let build_dir = working_dir
            .join(".stencila")
            .join("build")
            .join(Self::slugify_buildpack_id(buildpack_id));

        fs::create_dir_all(&build_dir).wrap_err(format!(
            "Could not create build plan directory `{}` for buildpack `{}`",
            build_dir.display(),
            buildpack_id.as_str()
        ))?;

        Ok(build_dir)
    }

    /// Create a CNB Build Plan file for a buildpack in a working directory
    ///
    /// Used in `detect` when the `build_plan` argument is not supplied.
    ///
    /// See https://github.com/buildpacks/spec/blob/main/buildpack.md#build-plan-toml
    fn build_plan_default(working_dir: &Path, buildpack_id: &BuildpackId) -> Result<PathBuf> {
        let dir = Self::build_dir_default(working_dir, buildpack_id)?;

        let build_plan_path = dir.join("build-plan.toml");
        if !build_plan_path.exists() {
            fs::File::create(&build_plan_path)?;
        }

        Ok(build_plan_path)
    }

    /// Create a CNB Buildpack Plan file for a buildpack in a working directory
    ///
    /// Used in `build` when the `buildpack_plan` argument is not supplied.
    ///
    /// See https://github.com/buildpacks/spec/blob/main/buildpack.md#buildpack-plan-toml
    fn buildpack_plan_default(working_dir: &Path, buildpack_id: &BuildpackId) -> Result<PathBuf> {
        let dir = Self::build_dir_default(working_dir, buildpack_id)?;

        // Get the `build-plan.toml`
        let build_plan_path = dir.join("build-plan.toml");
        let build_plan = if build_plan_path.exists() {
            let toml = fs::read_to_string(build_plan_path)?;
            toml::from_str::<BuildPlan>(&toml)?
        } else {
            BuildPlan::default()
        };

        // Write the `buildpack-plan.toml` using the plan's `requires` as `entries`
        let buildpack_plan_path = dir.join("buildpack-plan.toml");
        let buildpack_plan = BuildpackPlan {
            entries: build_plan.requires.unwrap_or_default(),
        };
        let toml = toml::to_string(&buildpack_plan)?;
        fs::write(&buildpack_plan_path, &toml)?;

        Ok(buildpack_plan_path)
    }

    /// Detect whether a buildpack should build a working directory
    ///
    /// # Arguments
    ///
    /// - `buildpack_label`: The label of the buildpack to detect with.
    ///
    /// - `working_dir`: The directory to detect; defaults to the current directory.
    ///
    /// - `platform_dir`: The platform directory for the platform (e.g. Pack or Stencila);
    ///                   defaults to `buildpacks/platform` in the Stencila app data directory.
    ///
    /// - `build_plan`: The path of the `plan.toml` file;
    ///                 defaults to `.stencila/build/<buildpack_label>/build-plan.toml` in the working directory.
    pub fn detect(
        &self,
        buildpack_id: &BuildpackId,
        working_dir: Option<&Path>,
        platform_dir: Option<&Path>,
        build_plan: Option<&Path>,
    ) -> Result<i32> {
        let current_dir = current_dir()?;
        let working_dir = working_dir.unwrap_or(&current_dir);
        let working_dir = working_dir.canonicalize().wrap_err(format!(
            "Could not find working directory `{}`",
            working_dir.display()
        ))?;

        let platform_dir = match platform_dir {
            Some(dir) => dir.to_owned(),
            None => Self::platform_dir()?,
        };

        let build_plan = match build_plan {
            Some(path) => path.to_owned(),
            None => Self::build_plan_default(&working_dir, buildpack_id)?,
        };

        set_current_dir(working_dir)?;
        let result = dispatch_builtins!(
            buildpack_id.as_str(),
            detect_with,
            &platform_dir,
            &build_plan
        );
        set_current_dir(current_dir)?;

        if let Ok(code) = result {
            if platform_dir_is_stencila(&platform_dir) && code != 0 {
                build_plan.parent().map(|dir| fs::remove_dir_all(dir).ok());
            }
        }

        result
    }

    /// Run `detect` for all buildpacks and return a map of the results
    pub fn detect_all(
        &self,
        working_dir: Option<&Path>,
        platform_dir: Option<&Path>,
    ) -> Result<Vec<(BuildpackId, bool)>> {
        let mut matched = Vec::new();

        for buildpack_toml in &self.inner {
            let buildpack_id = buildpack_toml.buildpack.id.clone();
            let result = self.detect(&buildpack_id, working_dir, platform_dir, None)?;
            matched.push((buildpack_id, result == 0));
        }

        Ok(matched)
    }

    /// Run `detect` for all buildpacks and compile the resulting `build-plan`.toml files into a
    /// map of `BuildPlan`s.
    ///
    /// The primary use for this method is to display to users what the build plan for their project
    /// is so that they can make changes (e.g. to `.tool-versions` or `requirements.txt` files).
    pub fn plan_all(
        &self,
        working_dir: Option<&Path>,
        platform_dir: Option<&Path>,
    ) -> Result<Vec<(BuildpackId, Option<BuildPlan>)>> {
        let working_dir = match working_dir {
            Some(dir) => dir.to_owned(),
            None => current_dir()?,
        };

        let matched = self.detect_all(Some(&working_dir), platform_dir)?;

        let build_plans = matched
            .into_iter()
            .map(|(buildpack_id, matched)| {
                let build_plan = match matched {
                    true => Self::build_plan_default(&working_dir, &buildpack_id)
                        .ok()
                        .and_then(|path| fs::read_to_string(path).ok())
                        .and_then(|toml| toml::from_str(&toml).ok()),
                    false => None,
                };
                (buildpack_id, build_plan)
            })
            .collect();

        Ok(build_plans)
    }

    /// Generate a Markdown document of a set of build plans
    pub fn plan_as_markdown(plans: &[(BuildpackId, Option<BuildPlan>)], show_all: bool) -> String {
        let plans = plans
            .iter()
            .map(|(id, plan)| {
                let md = match plan {
                    Some(plan) => plan
                        .requires
                        .iter()
                        .flatten()
                        .map(|require| {
                            let mut md = format!("- `{:18}`", require.name);
                            if let Some(metadata) = &require.metadata {
                                if let Some(desc) = metadata.get("description") {
                                    md.push_str(&format!(
                                        "  *{}*",
                                        desc.as_str().unwrap_or_default()
                                    ))
                                }
                                if let Some(source) = metadata.get("source") {
                                    md.push_str(&format!(
                                        " (`{}`)",
                                        source.as_str().unwrap_or_default()
                                    ))
                                }
                            }
                            md
                        })
                        .collect::<Vec<String>>()
                        .join("\n"),
                    None => "*Nothing*".to_string(),
                };
                if plan.is_some() || show_all {
                    format!("## Buildpack '{}'\n\n{}\n\n", id, md)
                } else {
                    "".to_string()
                }
            })
            .collect::<Vec<String>>()
            .concat();
        ["# Build plan\n\n", &plans].concat()
    }

    /// Build image layers for a working directory using a buildpack
    ///
    /// # Arguments
    ///
    /// - `buildpack_label`: The label of the buildpack to build with.
    ///
    /// - `working_dir`: The directory to build; defaults to the current directory.
    ///
    /// - `layers_dir`: A directory that may contain subdirectories representing each layer created
    ///                 by the buildpack in the final image or build cache;
    ///                 defaults to `.stencila/layers/<buildpack_label>` in the working directory.
    ///
    /// - `platform_dir`: The platform directory for the platform (e.g. Pack or Stencila);
    ///                   defaults to `buildpacks/platform` in the Stencila app data directory.
    ///
    /// - `buildpack_plan`: The path of the Buildpack Plan file;
    ///                      defaults to `.stencila/build/<buildpack_label>/buildpack-plan.toml` in the working directory.
    pub fn build(
        &self,
        buildpack_id: &BuildpackId,
        working_dir: Option<&Path>,
        layers_dir: Option<&Path>,
        platform_dir: Option<&Path>,
        buildpack_plan: Option<&Path>,
    ) -> Result<i32> {
        let current_dir = current_dir()?;
        let working_dir = working_dir.unwrap_or(&current_dir).canonicalize()?;

        tracing::info!(
            "Building `{}` with buildpack `{}`",
            working_dir.display(),
            buildpack_id
        );

        let layers_dir = match layers_dir {
            Some(dir) => dir.join(Self::slugify_buildpack_id(buildpack_id)),
            None => Self::layers_dir_default(&working_dir, buildpack_id)?,
        };

        let platform_dir = match platform_dir {
            Some(dir) => dir.to_owned(),
            None => Self::platform_dir()?,
        };

        let buildpack_plan = match buildpack_plan {
            Some(path) => path.to_owned(),
            None => Self::buildpack_plan_default(&working_dir, buildpack_id)?,
        };

        set_current_dir(working_dir)?;
        let result = dispatch_builtins!(
            buildpack_id.as_str(),
            build_with,
            &layers_dir,
            &platform_dir,
            &buildpack_plan
        );
        set_current_dir(current_dir)?;

        result
    }

    /// Prepare all buildpacks before a build
    ///
    /// This function is used to initialize the layers directory so that it contains a
    /// subdirectory for all buildpacks that create a layer (standard CNB buildpacks create a
    /// layer but some Stencila buildpacks e.g. `sources` and `dockerfile` do not). This
    /// is in turned used to record initial image snapshots for directories that may later change during
    /// the build process.
    pub fn prebuild_all(&self, layers_dir: &Path) -> Result<()> {
        for toml in &self.inner {
            if toml
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get("creates-layer"))
                .and_then(|value| value.as_bool())
                .unwrap_or(true)
            {
                let dir = layers_dir.join(Self::slugify_buildpack_id(&toml.buildpack.id));
                fs::create_dir_all(dir)?;
            }
        }

        Ok(())
    }

    /// Run `detect` for all buildpacks, run `build` for those that match, and return
    /// a map of the detection results.
    ///
    /// If the directory matches the `dockerfile` buildpack, no other buildpacks will
    /// be built for the directory.
    pub fn build_all(
        &self,
        working_dir: Option<&Path>,
        layers_dir: Option<&Path>,
        platform_dir: Option<&Path>,
    ) -> Result<Vec<(BuildpackId, bool)>> {
        let matches = self.detect_all(working_dir, platform_dir)?;

        let dockerfile_buildpack_id = buildpack_id!("stencila/dockerfile");
        for (buildpack_id, matched) in &matches {
            if *matched {
                self.build(buildpack_id, working_dir, layers_dir, platform_dir, None)?;
                if *buildpack_id == dockerfile_buildpack_id {
                    break;
                }
            }
        }

        Ok(matches)
    }

    /// Build a container image for a working directory
    pub async fn pack(&self, working_dir: Option<&Path>) -> Result<()> {
        let dockerfile_buildpack_id = buildpack_id!("stencila/dockerfile");
        let code = self
            .detect(&dockerfile_buildpack_id, working_dir, None, None)
            .unwrap_or(100);
        if code == 0 {
            self.build(&dockerfile_buildpack_id, working_dir, None, None, None)?;
            return Ok(());
        }

        let working_dir = match working_dir {
            Some(dir) => dir.to_owned(),
            None => current_dir()?,
        };

        let tag = tag_for_path(&working_dir);

        tracing::info!(
            "Building container image `{}` for `{}` with `pack` binary",
            tag,
            working_dir.display()
        );

        const CNB_BUILDER: &str = "stencila/builder:bionic";

        let pack = PackBinary {}.ensure().await?;
        pack.run(&[
            "build",
            &tag,
            "--builder",
            CNB_BUILDER,
            "--path",
            &working_dir.display().to_string(),
            "--pull-policy",
            "if-not-present",
            "--trust-builder",
        ])
        .await
    }

    /// Clean build directories for one, or all, buildpacks
    pub fn clean(&self, working_dir: Option<&Path>, buildpack: Option<&str>) -> Result<()> {
        let working_dir = match working_dir {
            Some(dir) => dir.to_owned(),
            None => current_dir()?,
        };

        let stencila_dir = working_dir.join(".stencila");
        let mut build_dir = stencila_dir.join("build");
        let mut layers_dir = stencila_dir.join("layers");

        if let Some(buildpack) = buildpack {
            if buildpack != "all" {
                build_dir.push(["stencila_", buildpack].concat());
                layers_dir.push(["stencila_", buildpack].concat());
            }
        }

        if build_dir.exists() {
            fs::remove_dir_all(build_dir)?;
        }
        if layers_dir.exists() {
            fs::remove_dir_all(layers_dir)?;
        }

        Ok(())
    }
}

impl Default for Buildpacks {
    fn default() -> Self {
        Self::new()
    }
}
