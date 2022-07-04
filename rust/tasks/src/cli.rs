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
    Analyze(Analyze),
    List(List),
    Run(Run_),
}

#[async_trait]
impl Run for Command {
    async fn run(&self) -> Result {
        match &self.action {
            Action::Analyze(action) => action.run().await,
            Action::List(action) => action.run().await,
            Action::Run(action) => action.run().await,
        }
    }
}

/// Reusable argument for the path of the Taskfile
#[derive(Parser)]
struct TaskfileArg {
    /// The Taskfile to use (defaults to the current)
    taskfile: Option<PathBuf>,
}

/// Analyze a directory contents and update its Taskfile
#[derive(Parser)]
pub struct Analyze {
    /// The directory to analyze
    ///
    /// If the directory does not yet have a Taskfile one will be created
    dir: Option<PathBuf>,
}

#[async_trait]
impl Run for Analyze {
    async fn run(&self) -> Result {
        Taskfile::analyze(self.dir.as_deref()).await?;
        result::nothing()
    }
}

/// List tasks in a Taskfile
///
/// Use this command to quickly get a list of all the tasks in a Taskfile.
#[derive(Parser)]
pub struct List {
    #[clap(flatten)]
    taskfile: TaskfileArg,
}

#[async_trait]
impl Run for List {
    async fn run(&self) -> Result {
        let taskfile = Taskfile::read(self.taskfile.taskfile.as_deref())?;
        result::table(
            taskfile.tasks.into_iter().map(TaskRow::from).collect_vec(),
            TaskRow::title(),
        )
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
