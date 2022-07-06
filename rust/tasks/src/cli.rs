use std::path::PathBuf;

use cli_utils::{
    clap::{self, Parser},
    common::async_trait::async_trait,
    result,
    table::{option_string, Table, Title},
    Result, Run,
};
use common::{itertools::Itertools, serde::Serialize};

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
    Refresh(Refresh),
    List(List),
    Run(Run_),
    Docs(Docs),
}

#[async_trait]
impl Run for Command {
    async fn run(&self) -> Result {
        match &self.action {
            Action::Refresh(action) => action.run().await,
            Action::List(action) => action.run().await,
            Action::Run(action) => action.run().await,
            Action::Docs(action) => action.run().await,
        }
    }
}

/// Reusable argument for the path of the Taskfile
#[derive(Parser)]
struct TaskfileArg {
    /// The Taskfile to use (defaults to the current)
    #[clap(short = 'f', long)]
    taskfile: Option<PathBuf>,
}

/// Refresh the Taskfile for a directory
#[derive(Parser)]
pub struct Refresh {
    /// The directory to refresh the Taskfile for
    ///
    /// If the directory does not yet have a Taskfile one will be created
    dir: Option<PathBuf>,
}

#[async_trait]
impl Run for Refresh {
    async fn run(&self) -> Result {
        Taskfile::refresh(self.dir.as_deref()).await?;
        result::nothing()
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
    taskfile: TaskfileArg,
}

#[async_trait]
impl Run for List {
    async fn run(&self) -> Result {
        let taskfile = Taskfile::read(self.taskfile.taskfile.as_deref())?;
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
                } else {
                    true
                }
            })
            .filter(|(name, ..)| {
                if let Some(action) = &self.action {
                    name.ends_with(&[":", action].concat())
                } else {
                    true
                }
            })
            .map(TaskRow::from)
            .collect_vec();
        result::table(tasks, TaskRow::title())
    }
}

/// Run a task in a Taskfile
///
/// Use this command to run one of the tasks in a Taskfile.
#[derive(Parser)]
pub struct Run_ {
    /// The name of the task to run
    task: String,

    #[clap(flatten)]
    taskfile: TaskfileArg,
}

#[async_trait]
impl Run for Run_ {
    async fn run(&self) -> Result {
        Taskfile::run(&self.task, self.taskfile.taskfile.as_deref()).await?;
        result::nothing()
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
