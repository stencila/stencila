use std::{
    fs,
    path::{Path, PathBuf},
};

use defaults::Defaults;
use eyre::{bail, eyre, Result};
use futures::future;
use graph_triples::{
    relations::{self, NULL_RANGE},
    resources, Resource, Triple,
};
use providers::provider::{ImportOptions, SyncOptions, WatchMode};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use stencila_schema::Node;
use strum::VariantNames;
use tokio::sync::mpsc;

/// A source-destination combination
///
/// Each source by destination combination should be unique to a project.
/// It is possible to have the same source being imported to multiple
/// destinations within a project and for multiple sources to used the same
/// destination (e.g. the root directory of the project).
#[skip_serializing_none]
#[derive(Clone, Debug, Defaults, Deserialize, Serialize)]
#[serde(default)]
pub struct Source {
    /// The name of the source
    ///
    /// Useful for providing a shorthand way to refer to the source rather than using it's URL
    pub name: Option<String>,

    /// The URL of the source
    pub url: String,

    /// The node parsed / detected from the URL
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

    /// A running cron task
    #[serde(skip)]
    #[def = "None"]
    cron_task: Option<SourceTask>,

    /// A running watch task
    #[serde(skip)]
    #[def = "None"]
    watch_task: Option<SourceTask>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SourceCron {
    /// The schedule on which to perform the action
    ///
    /// Can be a cron phrase (e.g. "every 10mins") or cron expression (e.g. "0 0 */10 * * *").
    schedule: String,

    /// The cron expression/s parsed from the `schedule`
    expressions: Vec<String>,

    /// The timezone parsed from the `schedule`
    timezone: String,

    /// The action to perform at each scheduled time
    action: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SourceWatch {
    /// The synchronization mode
    mode: Option<WatchMode>,
}

#[derive(Debug, Clone)]
struct SourceTask {
    /// The channel sender used to cancel the source task
    ///
    /// Note that if the source is dropped (ie. removed from a list of sources) with a running task
    /// then this canceller will be dropped and so the task will also end (an explicit `.send()`
    /// is not required).
    canceller: mpsc::Sender<()>,
}

impl Source {
    /// Create a new source
    pub async fn new(
        url: String,
        name: Option<String>,
        dest: Option<PathBuf>,
        cron: Option<String>,
        watch: Option<WatchMode>,
    ) -> Result<Source> {
        let node = Some(providers::resolve(&url).await?);

        let cron = if let Some(schedule) = cron {
            let (schedules, timezone) = cron_utils::parse(&schedule)?;
            let expressions = schedules
                .iter()
                .map(|schedule| schedule.to_string())
                .collect();
            let timezone = timezone.to_string();

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
            node,
            dest,
            cron,
            watch,
            ..Default::default()
        })
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

    /// Create a graph resource for the source
    pub fn resource(&self) -> Resource {
        resources::url(&self.url)
    }

    /// Create a set of graph triples describing relation between the source it's associated files.
    pub fn triples(&self, project: &Path) -> Vec<Triple> {
        let this = self.resource();
        self.files
            .iter()
            .map(|file| {
                (
                    this.clone(),
                    relations::imports(NULL_RANGE),
                    resources::file(&project.join(file)),
                )
            })
            .collect()
    }

    /// Import the source
    ///
    /// # Arguments
    ///
    /// - `path`: The path to import the source into
    pub async fn import(&self, path: &Path) -> Result<()> {
        let node = providers::resolve(&self.url).await?;
        let dest = match &self.dest {
            Some(dest) => path.join(dest),
            None => path.to_path_buf(),
        };
        let options = ImportOptions {
            secret_name: self.secret_name.clone(),
        };
        providers::import(&node, &dest, Some(options)).await?;

        Ok(())
    }

    /// Start cron and watch tasks (as applicable and as needed) for the source
    pub async fn start(&mut self, dest: &Path) -> Result<()> {
        self.cron_task_start(dest).await?;
        self.watch_task_start(dest).await?;

        Ok(())
    }

    /// Start a background cron task for the source
    ///
    /// A task has a `cron` spec and there is no tasks currently running for it.
    pub async fn cron_task_start(&mut self, dest: &Path) -> Result<()> {
        let cron = match (&self.cron, &self.cron_task) {
            (Some(cron), None) => cron,
            _ => return Ok(()),
        };

        tracing::info!("Starting cron task for source `{}`", self.label());
        let action = cron.action.clone().unwrap_or_default();
        let schedule = cron.schedule.clone();
        let node = providers::resolve(&self.url).await?;
        let dest = dest.to_path_buf();
        let (canceller, cancellee) = mpsc::channel(1);
        tokio::spawn(
            async move { providers::cron(&action, &schedule, &node, &dest, cancellee).await },
        );
        self.cron_task = Some(SourceTask { canceller });

        Ok(())
    }

    /// Start a background watch task for the source
    ///
    /// A task has a `watch` spec and there is no tasks currently running for it.
    pub async fn watch_task_start(&mut self, dest: &Path) -> Result<()> {
        let watch = match (&self.watch, &self.watch_task) {
            (Some(watch), None) => watch,
            _ => return Ok(()),
        };

        tracing::info!("Starting watch task for source `{}`", self.label());
        let node = providers::resolve(&self.url).await?;
        let dest = dest.to_path_buf();
        let (canceller, cancellee) = mpsc::channel(1);
        let options = SyncOptions {
            mode: watch.mode.clone(),
            secret_name: self.secret_name.clone(),
            ..Default::default()
        };
        tokio::spawn(async move { providers::watch(&node, &dest, cancellee, Some(options)).await });
        self.watch_task = Some(SourceTask { canceller });

        Ok(())
    }

    /// Start cron and watch tasks (if started) for the source
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(task) = &self.cron_task {
            tracing::info!("Stopping cron task for source `{}`", self.label());
            task.canceller.send(()).await?
        }
        self.cron_task = None;

        if let Some(task) = &self.watch_task {
            tracing::info!("Stopping watch task for source `{}`", self.label());
            task.canceller.send(()).await?
        }
        self.watch_task = None;

        Ok(())
    }
}

/// A set of sources, usually associated with a project
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Sources {
    /// The list of sources
    pub(crate) inner: Vec<Source>,

    /// If in a running state, the path that cron and watch tasks were started with
    ///
    /// If tasks are running then any new sources that are added will have
    /// tasks automatically started for them.
    #[serde(skip)]
    running: Option<PathBuf>,
}

impl Sources {
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
    pub async fn add(&mut self, mut source: Source) -> Result<()> {
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
        if let Some(path) = &self.running {
            source.start(path).await?;
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
            let mut removed = self.inner.remove(index);
            removed.stop().await?;
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
            tokio::spawn(async move { source.import(&path).await })
        });
        future::try_join_all(futures).await?;

        Ok(())
    }

    /// Start cron and/or watch tasks for each source
    pub async fn start(&mut self, path: &Path) -> Result<()> {
        for source in self.inner.iter_mut() {
            let dest = match &source.dest {
                Some(dest) => path.join(dest),
                None => path.to_path_buf(),
            };
            source.start(&dest).await?;
        }
        self.running = Some(path.to_path_buf());

        Ok(())
    }

    /// Stop any cron or watch tasks for each source
    pub async fn stop(&mut self) -> Result<()> {
        for source in self.inner.iter_mut() {
            source.stop().await?;
        }
        self.running = None;

        Ok(())
    }
}

#[cfg(feature = "cli")]
pub mod commands {
    use super::*;
    use crate::projects::PROJECTS;
    use async_trait::async_trait;
    use cli_utils::{result, Result, Run};
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Manage the a project's sources",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp,
        setting = structopt::clap::AppSettings::VersionlessSubcommands
    )]
    pub enum Command {
        List(List),
        Show(Show),
        Add(Add),
        Remove(Remove),
        Import(Import),
        Start(Start),
        Stop(Stop),
        Run(Run_),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            match self {
                Command::List(cmd) => cmd.run().await,
                Command::Show(cmd) => cmd.run().await,
                Command::Add(cmd) => cmd.run().await,
                Command::Remove(cmd) => cmd.run().await,
                Command::Import(cmd) => cmd.run().await,
                Command::Start(cmd) => cmd.run().await,
                Command::Stop(cmd) => cmd.run().await,
                Command::Run(cmd) => cmd.run().await,
            }
        }
    }

    /// List the sources for a project
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct List {
        /// The project to list sources for (defaults to the current project)
        project: Option<PathBuf>,
    }

    #[async_trait]
    impl Run for List {
        async fn run(&self) -> Result {
            let project = PROJECTS.open(self.project.clone(), false).await?;
            let sources = project.sources.list();

            result::value(sources)
        }
    }

    /// Show a source for a project
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Show {
        /// An identifier for the source
        source: String,

        /// The project that the source belongs to (defaults to the current project)
        project: Option<PathBuf>,
    }

    #[async_trait]
    impl Run for Show {
        async fn run(&self) -> Result {
            let project = PROJECTS.open(self.project.clone(), false).await?;
            let source = project.sources.find(&self.source)?;

            result::value(source)
        }
    }

    /// Add a source to a project
    ///
    /// Does not import the source use the `import` command for that.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Add {
        /// The URL (or "short URL" e.g github:owner/repo@v1.1) of the source to be added
        url: String,

        /// The path to import the source to
        dest: Option<PathBuf>,

        /// The project to add the source to (defaults to the current project)
        project: Option<PathBuf>,

        /// The name to give the source
        #[structopt(long, short)]
        name: Option<String>,

        /// A cron schedule for the source
        #[structopt(long, short)]
        cron: Option<String>,

        /// A watch mode for the source
        #[structopt(long, short, possible_values = WatchMode::VARIANTS)]
        watch: Option<WatchMode>,

        /// Parse the inputs into a source but do not add it to the project
        #[structopt(long)]
        dry_run: bool,
    }

    #[async_trait]
    impl Run for Add {
        async fn run(&self) -> Result {
            let project = PROJECTS.open(self.project.clone(), false).await?;
            let project = PROJECTS.get(project.path).await?;
            let mut project = project.lock().await;

            let source = Source::new(
                self.url.clone(),
                self.name.clone(),
                self.dest.clone(),
                self.cron.clone(),
                self.watch.clone(),
            )
            .await?;

            if self.dry_run {
                return result::value(source);
            }

            project.sources.add(source.clone()).await?;
            project.write().await?;

            tracing::info!("Added source to project");
            result::value(source)
        }
    }

    /// Remove a source from a project
    ///
    /// Note that this will remove all files imported from this source.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Remove {
        /// An identifier for the source
        source: String,

        /// The project to remove the source from (defaults to the current project)
        project: Option<PathBuf>,
    }

    #[async_trait]
    impl Run for Remove {
        async fn run(&self) -> Result {
            let project = PROJECTS.open(self.project.clone(), false).await?;
            let project = PROJECTS.get(project.path).await?;
            let mut project = project.lock().await;

            let source = project.sources.remove(&self.source).await?;
            project.write().await?;

            tracing::info!("Removed source from project");
            result::value(source)
        }
    }

    /// Import one or all of a project's sources
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Import {
        /// The project to remove the source from (defaults to the current project)
        project: Option<PathBuf>,

        /// An identifier for the source to import
        ///
        /// Only the first source matching this identifier will be imported.
        #[structopt(long, short)]
        source: Option<String>,
    }

    #[async_trait]
    impl Run for Import {
        async fn run(&self) -> Result {
            let project = PROJECTS.open(self.project.clone(), false).await?;
            if let Some(source) = &self.source {
                let source = project.sources.find(source)?;
                source.import(&project.path).await?;
                tracing::info!("Imported source `{}`", source.label());
            } else {
                project.sources.import(&project.path).await?;
                tracing::info!("Imported all sources");
            }

            result::nothing()
        }
    }

    /// Start cron and watch tasks for a project's sources
    ///
    /// This command is only useful in interactive mode because otherwise the
    /// process will exit straight away.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Start {
        /// The project to start tasks for (defaults to the current project)
        project: Option<PathBuf>,
    }

    #[async_trait]
    impl Run for Start {
        async fn run(&self) -> Result {
            let project = PROJECTS.open(self.project.clone(), false).await?;
            let path = project.path.clone();
            let project = PROJECTS.get(&path).await?;
            let mut project = project.lock().await;

            project.sources.start(&path).await?;

            result::nothing()
        }
    }

    /// Stop any cron and watch tasks for a project's sources
    ///
    /// This command is only useful in interactive mode. Use it to stop source tasks
    /// previously started using the `start` command.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Stop {
        /// The project to start tasks for (defaults to the current project)
        project: Option<PathBuf>,
    }

    #[async_trait]
    impl Run for Stop {
        async fn run(&self) -> Result {
            let project = PROJECTS.open(self.project.clone(), false).await?;
            let project = PROJECTS.get(project.path).await?;
            let mut project = project.lock().await;

            project.sources.stop().await?;

            result::nothing()
        }
    }

    /// Run cron and watch tasks for a project's sources
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Run_ {
        /// The project to run tasks for (defaults to the current project)
        project: Option<PathBuf>,
    }

    #[async_trait]
    impl Run for Run_ {
        async fn run(&self) -> Result {
            let project = PROJECTS.open(self.project.clone(), false).await?;
            let path = project.path.clone();
            let project = PROJECTS.get(&path).await?;
            let mut project = project.lock().await;

            // Start the sources
            project.sources.start(&path).await?;

            // Wait for interrupt signal
            let (subscriber, mut receiver) = mpsc::channel(1);
            events::subscribe_to_interrupt(subscriber).await;
            receiver.recv().await;

            // Stop the sources
            project.sources.stop().await?;

            result::nothing()
        }
    }
}
