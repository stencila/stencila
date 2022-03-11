use std::{
    fs,
    path::{Path, PathBuf},
};

use defaults::Defaults;
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use eyre::{bail, Result};
use futures::future;
use graph_triples::{
    relations::{self, NULL_RANGE},
    resources, Resource, Triple,
};

use providers::provider::{SyncMode, SyncOptions};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
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

    /// The destination path within the project
    pub dest: Option<PathBuf>,

    /// Run a cron schedule to import and/or export the source
    pub cron: Option<SourceCron>,

    /// Synchronize the source
    pub sync: Option<SourceSync>,

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

    /// A running sync task
    #[serde(skip)]
    #[def = "None"]
    sync_task: Option<SourceTask>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SourceCron {
    /// The schedule on which to perform the action
    ///
    /// A cron phrase (e.g. "every 10mins") or cron expression (e.g. "0 0 */10 * * *").
    schedule: String,

    /// The action to perform at each scheduled time
    action: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SourceSync {
    /// The synchronization mode
    mode: Option<SyncMode>,
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
    pub fn new(url: String, dest: Option<PathBuf>) -> Source {
        Source {
            url,
            dest,
            ..Default::default()
        }
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
        providers::import(&node, &dest, None).await?;

        Ok(())
    }

    /// Start cron and sync tasks (as applicable and as needed) for the source
    pub async fn start(&mut self, dest: &Path) -> Result<()> {
        self.cron_task_start(dest).await?;
        self.sync_task_start(dest).await?;

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

    /// Start a background sync task for the source
    ///
    /// A task has a `sync` spec and there is no tasks currently running for it.
    pub async fn sync_task_start(&mut self, dest: &Path) -> Result<()> {
        let sync = match (&self.sync, &self.sync_task) {
            (Some(sync), None) => sync,
            _ => return Ok(()),
        };

        tracing::info!("Starting sync task for source `{}`", self.label());
        let node = providers::resolve(&self.url).await?;
        let dest = dest.to_path_buf();
        let (canceller, cancellee) = mpsc::channel(1);
        let options = SyncOptions {
            token: self.token.clone(),
            mode: sync.mode.clone(),
            ..Default::default()
        };
        tokio::spawn(async move { providers::sync(&node, &dest, cancellee, Some(options)).await });
        self.sync_task = Some(SourceTask { canceller });

        Ok(())
    }

    /// Start cron and sync tasks (if started) for the source
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(task) = &self.cron_task {
            tracing::info!("Stopping cron task for source `{}`", self.label());
            task.canceller.send(()).await?
        }
        self.cron_task = None;

        if let Some(task) = &self.sync_task {
            tracing::info!("Stopping sync task for source `{}`", self.label());
            task.canceller.send(()).await?
        }
        self.sync_task = None;

        Ok(())
    }
}

/// A set of sources, usually associated with a
#[derive(Debug, Default, Clone, Deserialize, Serialize, AsRef, AsMut, Deref, DerefMut)]
pub struct Sources(Vec<Source>);

impl Sources {
    /// Return a list of sources
    pub fn list(&self) -> Vec<String> {
        self.iter().map(|source| source.label()).collect()
    }

    /// Find a source
    ///
    /// # Arguments
    ///
    /// - `identifier`: An identifier for a source.
    ///
    /// The first source that matches the identifier is returned. Will error if no match is found.
    pub fn find(&self, identifier: &str) -> Result<&Source> {
        if let Some(source) = self.iter().find(|source| source.matches(identifier)) {
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
    pub fn add(&mut self, source: Source) -> Result<()> {
        let source_dest = source.dest.clone().unwrap_or_else(|| PathBuf::from("."));
        for existing in self.iter() {
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
        self.as_mut().push(source);

        Ok(())
    }

    /// Remove a source
    ///
    /// # Arguments
    ///
    /// - `identifier`: An identifier for a source.
    ///
    /// The first source that matches the identifier is removed. See [`Source::matches`] for
    /// details. Will error if no match is found.
    pub fn remove(&mut self, identifier: &str) -> Result<Source> {
        if let Some(index) = self.iter().position(|source| source.matches(identifier)) {
            let removed = self.as_mut().remove(index);
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
        let futures = self.iter().map(|source| {
            let source = source.clone();
            let path = path.to_path_buf();
            tokio::spawn(async move { source.import(&path).await })
        });
        future::try_join_all(futures).await?;

        Ok(())
    }

    /// Start cron and/or sync tasks for each source
    pub async fn start(&mut self, path: &Path) -> Result<()> {
        for source in self.iter_mut() {
            let dest = match &source.dest {
                Some(dest) => path.join(dest),
                None => path.to_path_buf(),
            };
            source.start(&dest).await?;
        }

        Ok(())
    }

    /// Stop any cron or sync tasks for each source
    pub async fn stop(&mut self) -> Result<()> {
        for source in self.iter_mut() {
            source.stop().await?;
        }

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
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
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
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
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
    }

    #[async_trait]
    impl Run for Add {
        async fn run(&self) -> Result {
            let project = PROJECTS.open(self.project.clone(), false).await?;
            let project = PROJECTS.get(project.path).await?;
            let mut project = project.lock().await;

            let source = Source {
                name: self.name.clone(),
                url: self.url.clone(),
                dest: self.dest.clone(),
                ..Default::default()
            };
            project.sources.add(source.clone())?;
            project.write().await?;

            tracing::info!("Added source");
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

            let source = project.sources.remove(&self.source)?;
            project.write().await?;

            tracing::info!("Removed source");
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

    /// Start cron and sync tasks for a project's sources
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

    /// Stop any cron and sync tasks for a project's sources
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

    /// Run cron and sync tasks for a project's sources
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
