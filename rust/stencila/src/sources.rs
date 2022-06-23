use std::path::PathBuf;

use common::{serde_json, tokio::sync::mpsc, tracing};
use providers::provider::WatchMode;
use sources::Source;

#[cfg(feature = "cli")]
pub mod commands {
    use cli_utils::{
        clap::{self, Parser},
        result, Result, Run,
    };
    use common::{async_trait::async_trait, strum::VariantNames, tempfile};

    use crate::projects::PROJECTS;

    use super::*;

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
        Import(Import),
        Start(Start),
        Stop(Stop),
        Run(Run_),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            match &self.action {
                Action::List(action) => action.run().await,
                Action::Show(action) => action.run().await,
                Action::Add(action) => action.run().await,
                Action::Remove(action) => action.run().await,
                Action::Import(action) => action.run().await,
                Action::Start(action) => action.run().await,
                Action::Stop(action) => action.run().await,
                Action::Run(action) => action.run().await,
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
            let project = PROJECTS.open(self.project.clone(), false).await?;
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
            let project = PROJECTS.open(self.project.clone(), false).await?;
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

            let temp_dir = tempfile::tempdir()?;
            let path = match self.dry_run {
                true => temp_dir.path(),
                false => &project.path,
            };
            let files = source.import(path).await?;

            if !self.dry_run {
                project.sources.add(source.clone()).await?;
                project.write().await?;
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
    #[derive(Parser)]
    pub struct Import {
        /// The project to import the source into (defaults to the current project)
        project: Option<PathBuf>,

        /// An identifier for the source to import
        ///
        /// Only the first source matching this identifier will be imported.
        #[clap(long, short)]
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
    #[derive(Parser)]
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
    #[derive(Parser)]
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
    #[derive(Parser)]
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
