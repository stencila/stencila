use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use common::{
    defaults::Defaults,
    eyre::{bail, Result},
    futures::future,
    serde::{Deserialize, Serialize},
    serde_json,
    serde_with::skip_serializing_none,
    strum::{Display, EnumString, EnumVariantNames, VariantNames},
    tracing,
};
use files::File;
// use providers::provider::{ImportOptions, WatchMode};
use stencila_schema::Node;

use crate::types::ProjectLocal;

/// A source-destination combination
///
/// Each source by destination combination should be unique to a project.
/// It is possible to have the same source being imported to multiple
/// destinations within a project and for multiple sources to used the same
/// destination (e.g. the root directory of the project).
#[skip_serializing_none]
#[derive(Clone, Debug, Defaults, Deserialize, Serialize)]
#[serde(default, crate = "common::serde")]
pub struct Source {
    /// The name of the source
    ///
    /// Useful for providing a shorthand way to refer to the source rather than using it's URL
    pub name: Option<String>,

    /// The URL of the source
    pub url: String,

    /// The providers that matched the URL
    pub provider: Option<String>,

    /// The node parsed from the URL by the provider
    pub node: Option<Node>,

    /// The destination path within the project
    pub dest: Option<PathBuf>,

    /// Run a cron schedule to import and/or export the source
    pub cron: Option<SourceCron>,

    /// Synchronize the source
    pub watch: Option<SourceWatch>,

    /// The name of the secret required to access the source
    ///
    /// To improve the security of API access tokens, secrets are only ever read from
    /// environment variables. Source providers usually have a default secret name
    /// e.g. `GITHUB_TOKEN`. However, this field allows setting of custom secret names
    /// which may be necessary, for example, if a project uses two sources from the
    /// same provider, requiring different secrets.
    pub secret_name: Option<String>,

    /// A list of file paths currently associated with the source (relative to the project root)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<PathBuf>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, crate = "common::serde")]
pub struct SourceCron {
    /// The schedule on which to perform the action
    ///
    /// Can be a cron phrase (e.g. "every 10mins") or cron expression (e.g. "0 0 */10 * * *").
    schedule: String,

    /// The cron expression/s parsed from the `schedule`
    expressions: Option<Vec<String>>,

    /// The timezone parsed from the `schedule`
    timezone: Option<String>,

    /// The action to perform at each scheduled time
    action: Option<String>,
}

// TODO: This, and `todo!()`s below are temporary, pending moving these into their own crate.
#[derive(Debug, Clone, Deserialize, Serialize, Display, EnumString, EnumVariantNames)]
#[serde(rename_all = "lowercase", crate = "common::serde")]
#[strum(crate = "common::strum")]
pub enum WatchMode {}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(crate = "common::serde")]
pub struct SourceWatch {
    /// The synchronization mode
    mode: Option<WatchMode>,
}

impl Source {
    /// Create a new source
    ///
    /// Resolves a [`Provider`] for the source.
    /// Ensures that `dest` is a relative path and does not include any traversal (i.e. `..`).
    /// Parses the `cron` into an expression and timezone.
    pub async fn new(
        _url: String,
        _name: Option<String>,
        _dest: Option<PathBuf>,
        _cron: Option<String>,
        _watch: Option<WatchMode>,
    ) -> Result<Source> {
        todo!("Reimplement in separate crate?");

        /*
        let (provider, node) = providers::resolve(&url).await?;

        if let Some(dest) = dest.as_ref() {
            if dest.is_absolute() {
                bail!("Source destination must be a relative path; try removing any leading slash (MacOS or Linux) or drive letter (e.g. `C:/` on Windows)")
            }
            if dest.to_string_lossy().contains("..") {
                bail!("Source destination must not have any path traversal (a.k.a directory climbing); try removing any `..` in the destination path")
            }
        }

        let cron = if let Some(schedule) = cron {
            let (schedules, timezone) = cron_utils::parse(&schedule)?;
            let (expressions, timezone) = if !schedules.is_empty() {
                (
                    Some(
                        schedules
                            .iter()
                            .map(|schedule| schedule.to_string())
                            .collect(),
                    ),
                    timezone.map(|tz| tz.to_string()),
                )
            } else {
                (None, None)
            };

            Some(SourceCron {
                schedule,
                expressions,
                timezone,
                action: None,
            })
        } else {
            None
        };

        let watch = watch.map(|mode| SourceWatch { mode: Some(mode) });

        Ok(Source {
            name,
            url,
            provider: Some(provider),
            node: Some(node),
            dest,
            cron,
            watch,
            ..Default::default()
        })
        */
    }

    /// Generate a label for the source
    pub fn label(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.url.clone())
    }

    /// Does the source match a string identifier
    ///
    /// Matches the string against the `name`, the `url` , or the filename (of the `dest` with or without extension).
    pub fn matches(&self, identifier: &str) -> bool {
        if let Some(name) = &self.name {
            if name == identifier {
                return true;
            }
        }

        if identifier == self.url {
            return true;
        }

        if let Some(dest) = &self.dest {
            if dest.to_string_lossy() == identifier {
                return true;
            }
            if let Some(file_stem) = dest.file_stem() {
                if file_stem == identifier {
                    return true;
                }
            }
        }

        false
    }

    /// Read a source from a JSON file
    ///
    /// Only changes the properties that are NOT saved in the project.json file.
    pub fn read<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            bail!("Project source file does not exist: {}", path.display())
        }
        let path = path.canonicalize()?;

        let json = fs::read_to_string(path)?;
        let source: Source = serde_json::from_str(&json)?;
        self.files = source.files;

        Ok(())
    }

    /// Import the source
    ///
    /// # Arguments
    ///
    /// - `path`: The path to import the source into
    ///
    /// # Returns
    ///
    /// A map of the [`File`]s that were imported by the source.
    pub async fn import(&self, _path: &Path) -> Result<BTreeMap<PathBuf, File>> {
        todo!("Reimplement in separate crate?");

        /*
        let (.., node) = providers::resolve(&self.url).await?;
        let dest = match &self.dest {
            Some(dest) => path.join(dest),
            None => path.to_path_buf(),
        };
        let options = ImportOptions {
            token: self.secret_name.clone(),
        };

        let (pre, ..) = Files::walk(path, false);

        providers::import(&node, &dest, Some(options)).await?;

        let (post, ..) = Files::walk(path, false);
        let files = post
            .into_iter()
            .filter_map(|(path, file)| match pre.get(&path) {
                None => Some((path, file)),
                Some(existing) => {
                    if existing.size != file.size && existing.modified != file.modified {
                        Some((path, file))
                    } else {
                        None
                    }
                }
            });

        Ok(files.collect())
        */
    }
}

/// A set of sources, usually associated with a project
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(transparent, crate = "common::serde")]
pub struct Sources {
    /// The list of sources
    pub inner: Vec<Source>,
}

impl Sources {
    /// Are there any sources?
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Return a list of sources
    pub fn list(&self) -> Vec<String> {
        self.inner.iter().map(|source| source.label()).collect()
    }

    /// Find a source
    ///
    /// # Arguments
    ///
    /// - `identifier`: An identifier for a source.
    ///
    /// The first source that matches the identifier is returned. Will error if no match is found.
    pub fn find(&self, identifier: &str) -> Result<&Source> {
        if let Some(source) = self.inner.iter().find(|source| source.matches(identifier)) {
            return Ok(source);
        }

        bail!("Unable to find a source matching `{}`", identifier)
    }

    /// Add a source
    ///
    /// # Arguments
    ///
    /// - `source`: The source to add
    ///
    /// Warns if there is already a source with the same `url`.
    /// Errors if there is already a source with the same `name` or the same `url` and `dest`.
    ///
    /// If the `start` method has been called previously then the sources are in a "running"
    /// state and the `start` will be called on the source.
    pub async fn add(&mut self, source: Source) -> Result<()> {
        let source_dest = source.dest.clone().unwrap_or_else(|| PathBuf::from("."));
        for existing in self.inner.iter() {
            if let (Some(existing_name), Some(source_name)) = (&existing.name, &source.name) {
                if existing_name == source_name {
                    bail!("There is already a source with the name `{}`", source_name);
                }
            }
            let existing_dest = existing.dest.clone().unwrap_or_else(|| PathBuf::from("."));
            if existing.url == source.url && existing_dest == source_dest {
                bail!(
                    "There is already a source with the URL `{}` and destination `{}`",
                    source.url,
                    source_dest.display()
                );
            }
            if existing.url == source.url {
                tracing::warn!("There is already a source with the URL `{}`", source.url);
            }
        }
        self.inner.push(source);

        Ok(())
    }

    /// Remove a source
    ///
    /// # Arguments
    ///
    /// - `identifier`: An identifier for a source.
    ///
    /// Any running tasks for the source will be stopped.
    ///
    /// The first source that matches the identifier is removed. See [`Source::matches`] for
    /// details. Will error if no match is found.
    pub async fn remove(&mut self, identifier: &str) -> Result<Source> {
        if let Some(index) = self
            .inner
            .iter()
            .position(|source| source.matches(identifier))
        {
            let removed = self.inner.remove(index);
            return Ok(removed);
        }

        bail!("Unable to find a matching source to remove")
    }

    /// Import all sources
    ///
    /// # Arguments
    ///
    /// - `path`: The path to import the sources into (usually a project directory)
    ///
    /// Imports sources in parallel.
    pub async fn import(&self, path: &Path) -> Result<()> {
        let futures = self.inner.iter().map(|source| {
            let source = source.clone();
            let path = path.to_path_buf();
            async move { source.import(&path).await }
        });
        future::try_join_all(futures).await?;

        Ok(())
    }
}

pub mod cli {
    use super::*;

    use cli_utils::{
        clap::{self, Parser},
        result, Result, Run,
    };
    use common::{async_trait::async_trait, serde_json, tempfile, tracing};

    /// Manage and use project sources
    #[derive(Parser)]
    pub struct Command {
        #[clap(subcommand)]
        pub action: Action,
    }

    #[derive(Parser)]
    pub enum Action {
        List(List),
        Show(Show),
        Add(Add),
        Remove(Remove),
        Pull(Pull),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            match &self.action {
                Action::List(action) => action.run().await,
                Action::Show(action) => action.run().await,
                Action::Add(action) => action.run().await,
                Action::Remove(action) => action.run().await,
                Action::Pull(action) => action.run().await,
            }
        }
    }

    /// List the sources for a project
    #[derive(Parser)]
    pub struct List {
        /// The project to list sources for (defaults to the current project)
        project: Option<PathBuf>,
    }

    #[async_trait]
    impl Run for List {
        async fn run(&self) -> Result {
            let project = ProjectLocal::current()?;
            let sources = project.sources.list();

            result::value(sources)
        }
    }

    /// Show a source for a project
    #[derive(Parser)]
    pub struct Show {
        /// An identifier for the source
        source: String,

        /// The project that the source belongs to (defaults to the current project)
        project: Option<PathBuf>,
    }

    #[async_trait]
    impl Run for Show {
        async fn run(&self) -> Result {
            let project = ProjectLocal::current()?;
            let source = project.sources.find(&self.source)?;

            result::value(source)
        }
    }

    /// Add a source to a project
    ///
    /// Does not import the source use the `import` command for that.
    #[derive(Parser)]
    pub struct Add {
        /// The URL (or "short URL" e.g github:owner/repo@v1.1) of the source to be added
        url: String,

        /// The path to import the source to
        dest: Option<PathBuf>,

        /// The project to add the source to (defaults to the current project)
        project: Option<PathBuf>,

        /// The name to give the source
        #[clap(long, short)]
        name: Option<String>,

        /// A cron schedule for the source
        #[clap(long, short)]
        cron: Option<String>,

        /// A watch mode for the source
        #[clap(long, short, possible_values = WatchMode::VARIANTS)]
        watch: Option<WatchMode>,

        /// Do a dry run of adding the source
        ///
        /// Parses the input URL and other arguments into a source but does not add it, or the
        /// files that it imports, to the project. Useful for checking URL and cron formats
        /// and previewing the files that will be imported.
        #[clap(long)]
        dry_run: bool,
    }

    #[async_trait]
    impl Run for Add {
        async fn run(&self) -> Result {
            let mut project = ProjectLocal::current()?;

            let source = Source::new(
                self.url.clone(),
                self.name.clone(),
                self.dest.clone(),
                self.cron.clone(),
                self.watch.clone(),
            )
            .await?;

            let temp_dir = tempfile::tempdir()?;
            let path = match self.dry_run {
                true => temp_dir.path(),
                false => project.dir(),
            };
            let files = source.import(path).await?;

            if !self.dry_run {
                project.sources.add(source.clone()).await?;
                project.write()?;
            }

            tracing::info!("Added source to project");
            result::value(serde_json::json!({
                "source": source,
                "files": files
            }))
        }
    }

    /// Remove a source from a project
    ///
    /// Note that this will remove all files imported from this source.
    #[derive(Parser)]
    pub struct Remove {
        /// An identifier for the source
        source: String,

        /// The project to remove the source from (defaults to the current project)
        project: Option<PathBuf>,
    }

    #[async_trait]
    impl Run for Remove {
        async fn run(&self) -> Result {
            let mut project = ProjectLocal::current()?;
            let source = project.sources.remove(&self.source).await?;
            project.write()?;

            tracing::info!("Removed source from project");
            result::value(source)
        }
    }

    /// Pull one or all of a project's sources
    #[derive(Parser)]
    pub struct Pull {
        /// The project to import the source into (defaults to the current project)
        project: Option<PathBuf>,

        /// An identifier for the source to import
        ///
        /// Only the first source matching this identifier will be imported.
        #[clap(long, short)]
        source: Option<String>,
    }

    #[async_trait]
    impl Run for Pull {
        async fn run(&self) -> Result {
            let project = ProjectLocal::current()?;
            if let Some(source) = &self.source {
                let source = project.sources.find(source)?;
                source.import(project.dir()).await?;
                tracing::info!("Imported source `{}`", source.label());
            } else {
                project.sources.import(project.dir()).await?;
                tracing::info!("Imported all sources");
            }

            result::nothing()
        }
    }
}
