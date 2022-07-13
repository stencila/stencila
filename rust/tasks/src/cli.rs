use std::path::PathBuf;

use cli_utils::{
    clap::{self, Parser},
    common::async_trait::async_trait,
    result,
    table::{option_string, Table, Title},
    Result, Run,
};
use common::{eyre::bail, itertools::Itertools, serde::Serialize};

use crate::taskfile::{Task, Taskfile};

/// Manage and run project tasks
#[derive(Parser)]
#[clap(alias = "task")]
pub struct Command {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Parser)]
pub enum Action {
    Init(Init),
    List(List),
    Run(Run_),
    Detect(Detect),
    Docs(Docs),
}

#[async_trait]
impl Run for Command {
    async fn run(&self) -> Result {
        match &self.action {
            Action::Init(action) => action.run().await,
            Action::List(action) => action.run().await,
            Action::Run(action) => action.run().await,
            Action::Detect(action) => action.run().await,
            Action::Docs(action) => action.run().await,
        }
    }
}

/// Reusable argument for the path of the Taskfile
#[derive(Parser)]
struct TaskfileOption {
    /// The Taskfile to use (defaults to the current)
    #[clap(short = 'f', long)]
    taskfile: Option<PathBuf>,
}

/// Initialize a tasks for a directory
#[derive(Parser)]
pub struct Init {
    /// The directory to initialize tasks for
    ///
    /// If the directory does not yet have a Taskfile one will be created
    dir: Option<PathBuf>,
}

#[async_trait]
impl Run for Init {
    async fn run(&self) -> Result {
        let taskfile = Taskfile::init(self.dir.as_deref(), 0).await?;
        result::value(taskfile)
    }
}

/// List tasks in a Taskfile
///
/// Use this command to quickly get a list of all the tasks in a Taskfile.
#[derive(Parser)]
pub struct List {
    /// List all tasks, including those in included Taskfiles
    ///
    /// By default only task that are defined in the root Taskfile are listed.
    /// Use this option to show all tasks, including those from included Taskfiles.
    #[clap(short, long)]
    all: bool,

    /// Filter tasks by topic e.g. 'python', 'git'
    #[clap(short, long)]
    topic: Option<String>,

    /// Filter tasks by action e.g. 'add', 'remove'
    #[clap(short = 'c', long)]
    action: Option<String>,

    #[clap(flatten)]
    taskfile: TaskfileOption,
}

#[async_trait]
impl Run for List {
    async fn run(&self) -> Result {
        let taskfile = Taskfile::init(self.taskfile.taskfile.as_deref(), 2).await?;
        let tasks = taskfile
            .tasks
            .into_iter()
            .filter(|(.., task)| !task.hide)
            .filter(
                |(name, ..)| {
                    if self.all {
                        true
                    } else {
                        !name.contains(':')
                    }
                },
            )
            .filter(|(name, ..)| {
                if let Some(topic) = &self.topic {
                    name.starts_with(&[topic, ":"].concat())
                        || name.starts_with(&["lib:", topic, ":"].concat())
                } else {
                    true
                }
            })
            .filter(|(name, ..)| {
                if let Some(action) = &self.action {
                    name == action || name.ends_with(&[":", action].concat())
                } else {
                    true
                }
            })
            .map(TaskRow::from)
            .collect_vec();
        result::table(tasks, TaskRow::title())
    }
}

#[derive(Default, Serialize, Table)]
#[serde(crate = "common::serde")]
#[table(crate = "cli_utils::cli_table")]
struct TaskRow {
    #[table(title = "Name")]
    name: String,

    #[table(title = "Description", display_fn = "option_string")]
    desc: Option<String>,
}

impl From<(String, Task)> for TaskRow {
    fn from((name, task): (String, Task)) -> Self {
        TaskRow {
            name,
            desc: task.desc,
        }
    }
}

/// Run a task in a Taskfile
///
/// Use this command to run one of the tasks in a Taskfile.
#[derive(Parser)]
pub struct Run_ {
    /// The names and variables of the tasks to run
    #[clap(required = true)]
    tasks: Vec<String>,

    /// Run the tasks once, immediately and ignore `schedule` or `watches`
    #[clap(short, long, conflicts_with_all = &["schedule", "watch"])]
    now: bool,

    /// Run the tasks on a time schedule
    #[clap(short, long)]
    schedule: Option<String>,

    /// Run the tasks when files matching this pattern change
    #[clap(short, long)]
    watch: Option<String>,

    /// Ignore changes to files matching this pattern
    #[clap(long)]
    ignore: Option<String>,

    /// Number of seconds to delay running tasks after file changes
    #[clap(short, long)]
    delay: Option<u64>,

    #[clap(flatten)]
    taskfile: TaskfileOption,

    /// An internal, hidden option used to contextualize error messages
    /// when used as a fallback command
    #[clap(long, hide = true)]
    error_prefix: Option<String>,
}

#[async_trait]
impl Run for Run_ {
    async fn run(&self) -> Result {
        match Taskfile::run(
            &self.tasks,
            self.taskfile.taskfile.as_deref(),
            self.now,
            self.schedule.as_deref(),
            self.watch.as_deref(),
            self.ignore.as_deref(),
            self.delay,
        )
        .await
        {
            Ok(..) => result::nothing(),
            Err(error) => match &self.error_prefix {
                Some(prefix) => bail!("{} {}", prefix, error.to_string()),
                None => Err(error),
            },
        }
    }
}

/// Detect which tasks are required by a project
///
/// This command is equivalent to `stencila tasks run detect`. It is mostly used
/// internally as a "callback" to update the Taskfile at the end of the `detect`
/// task with the `--no-run` option to avoid recursively running the task.
#[derive(Parser)]
pub struct Detect {
    #[clap(flatten)]
    taskfile: TaskfileOption,

    /// Do not run the `detect` task, only read the detect tasks produced by
    /// a previous run of that task.
    #[clap(long)]
    no_run: bool,
}

#[async_trait]
impl Run for Detect {
    async fn run(&self) -> Result {
        let mut taskfile = Taskfile::init(self.taskfile.taskfile.as_deref(), 0).await?;
        taskfile.detect(!self.no_run).await?;
        result::nothing()
    }
}

/// Generate docs for Taskfiles
///
/// This is currently hidden but in the future may be exposed so users
/// can generate docs for their own Taskfiles.
#[derive(Parser)]
#[clap(hide = true)]
pub struct Docs;

#[async_trait]
impl Run for Docs {
    async fn run(&self) -> Result {
        Taskfile::docs_all()?;
        result::nothing()
    }
}
