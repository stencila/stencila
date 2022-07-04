use std::{
    env::current_dir,
    fs::{create_dir_all, read_to_string, write},
    path::{Path, PathBuf},
};

use binary_task::{BinaryTrait, TaskBinary};
use common::{
    defaults::Defaults,
    eyre::{bail, eyre, Result},
    indexmap::IndexMap,
    itertools::Itertools,
    once_cell::sync::Lazy,
    regex::Regex,
    serde::{Deserialize, Deserializer, Serialize},
    serde_with::skip_serializing_none,
    serde_yaml, tracing,
};
use rust_embed::RustEmbed;

#[skip_serializing_none]
#[derive(Defaults, Deserialize, Serialize)]
#[serde(default, crate = "common::serde")]
pub struct Taskfile {
    /// The version of the Taskfile schema
    #[def = "\"3\".to_string()"]
    pub version: String,

    /// Additional Taskfiles to be included
    ///
    /// See https://taskfile.dev/usage/#including-other-taskfiles.
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub includes: IndexMap<String, Include>,

    /// The mode for controlling task output
    ///
    /// Available options: `interleaved` (default), `group` and `prefixed`.
    /// See https://taskfile.dev/usage/#output-syntax.
    pub output: Option<String>,

    /// Method for determining the status of tasks
    ///
    /// Available options: `checksum` (default), `timestamp` and none.
    /// Can be overridden on a task by task basis.
    /// See https://taskfile.dev/usage/#prevent-unnecessary-work.
    pub method: Option<String>,

    /// Do not print task execution lines
    ///
    /// Defaults to `false`. If `false`, can be overridden with `true` on a task by task basis.
    /// Note that `stdout` and `stderr` of command will always be shown.
    /// See https://taskfile.dev/usage/#silent-mode.
    #[serde(skip_serializing_if = "is_false")]
    pub silent: bool,

    /// Whether the task should be run again or not if called more than once
    ///
    /// Available options: `always` (default), `once` and `when_changed`.
    /// Can be overridden on a task by task basis.
    pub run: Option<String>,

    /// Variables that can be used in all tasks
    ///
    /// See https://taskfile.dev/usage/#variables.
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub vars: IndexMap<String, Variable>,

    /// Environment variables used for all tasks
    ///
    /// See https://taskfile.dev/usage/#task.
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub env: IndexMap<String, Variable>,

    /// Environment variable files to be used for all tasks
    ///
    /// See https://taskfile.dev/usage/#env-files.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dotenv: Vec<String>,

    /// Task definitions
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub tasks: IndexMap<String, Task>,
}

impl Taskfile {
    /// Resolve a Taskfile for a path
    fn resolve(path: Option<&Path>) -> Result<PathBuf> {
        Ok(match path {
            Some(path) => {
                if path.is_dir() {
                    path.join("Taskfile.yaml")
                } else {
                    path.into()
                }
            }
            None => current_dir()?.join("Taskfile.yaml"),
        })
    }

    /// Load a Taskfile from a YAML string
    fn load(yaml: &str) -> Result<Self> {
        let taskfile = serde_yaml::from_str(yaml)?;
        Ok(taskfile)
    }

    /// Read a Taskfile from a filesystem path
    ///
    /// In addition to deserializing the Taskfile, will resolve its `includes`
    /// and add those to the task list.
    pub fn read(path: Option<&Path>) -> Result<Self> {
        let path = Self::resolve(path)?;
        let yaml = read_to_string(&path)?;

        let mut taskfile = match Self::load(&yaml) {
            Ok(taskfile) => taskfile,
            Err(error) => bail!("While reading Taskfile `{}`: {}", path.display(), error),
        };

        let dir = path.parent().expect("Should always have a parent");
        for (include_name, include) in &taskfile.includes {
            let include_path = dir.join(&include.taskfile);
            let include_path = Self::resolve(Some(&include_path))?;
            if !include.optional && !include_path.exists() {
                bail!(
                    "Included Taskfile does not exist: {}",
                    include_path.display()
                )
            };

            let include_taskfile = Self::read(Some(&include_path))?;
            for (task_name, task) in include_taskfile.tasks {
                // Do not include tasks that themselves were included
                if !task_name.contains(':') {
                    taskfile
                        .tasks
                        .insert([include_name, ":", &task_name].concat(), task);
                }
            }
        }

        Ok(taskfile)
    }

    /// Dump the Taskfile to a YAML string
    ///
    /// Attempts to conform to the Taskfile style guide by placing new
    /// lines before root properties and tasks.
    /// See https://taskfile.dev/styleguide/.
    fn dump(&self) -> Result<String> {
        let yaml = serde_yaml::to_string(self)?;
        let yaml = yaml.strip_prefix("---\n").unwrap_or(yaml.as_str());

        static PROP_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new("(?m)^[a-z]+:").expect("Unable to create regex"));
        static TASK_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new("(?m)^  [\\w\\-_]+:").expect("Unable to create regex"));

        let mut task_count = -1;
        let yaml = yaml
            .lines()
            .into_iter()
            .map(|line| {
                if PROP_REGEX.is_match(line) {
                    if line.starts_with("tasks:") {
                        task_count = 0;
                    }
                    ["\n", line].concat()
                } else if task_count >= 0 && TASK_REGEX.is_match(line) {
                    task_count += 1;
                    if task_count > 1 {
                        ["\n", line].concat()
                    } else {
                        line.to_string()
                    }
                } else {
                    line.to_string()
                }
            })
            .collect_vec()
            .join("\n")
            .trim_start()
            .to_string();

        Ok(yaml)
    }

    /// Write the Taskfile to a filesystem path
    pub fn write(&self, path: Option<&Path>) -> Result<()> {
        let yaml = self.dump()?;
        let path = Self::resolve(path).or_else(|_| current_dir())?;
        write(path, yaml)?;
        Ok(())
    }

    /// Analyze the content of the project and add tasks to its Taskfile
    pub async fn analyze(dir: Option<&Path>) -> Result<()> {
        let dir = match dir {
            Some(dir) => dir.to_path_buf(),
            None => current_dir()?,
        };

        let path = dir.join("Taskfile.yaml");
        let mut taskfile = match path.exists() {
            true => Taskfile::read(Some(&path))?,
            false => Taskfile::default(),
        };

        // Remove any existing tasks `autogen` includes and tasks
        // If this is not done they may accumulate even if not needed
        taskfile.includes.retain(|_name, include| !include.autogen);
        taskfile.tasks.retain(|_name, task| !task.autogen);

        // TODO: add tasks needed based on dependencies

        taskfile.write(Some(&path))
    }

    /// Run a task in the Taskfile
    pub async fn run(task: &str, path: Option<&Path>) -> Result<()> {
        let path = Self::resolve(path)?;

        // Check that task is in taskfile. This will be done by the `task` binary
        // but doing it here is a fail early and produces a more explicit error message
        let taskfile = Taskfile::read(Some(&path))?;
        if !taskfile.tasks.contains_key(task) {
            bail!("Task `{}` not found in Taskfile `{}`", task, path.display());
        }

        let binary = TaskBinary {}.ensure().await?;
        binary
            .run_with(
                [task, "--taskfile", &path.display().to_string()],
                Some(tracing::Level::INFO),
                Some(tracing::Level::INFO),
            )
            .await?;

        Ok(())
    }
}

#[derive(Clone, Defaults, Deserialize, Serialize)]
#[serde(
    from = "IncludeSyntax",
    into = "IncludeSyntax",
    crate = "common::serde"
)]
pub struct Include {
    /// The path for the Taskfile or directory to be included
    ///
    /// If a directory, Task will look for files named Taskfile.yml or Taskfile.yaml inside that directory.
    taskfile: String,

    /// The working directory of the included tasks when run
    ///
    /// Defaults to the parent Taskfile directory.
    dir: Option<String>,

    /// Whether the included Taskfile is optional
    ///
    /// If true, no errors will be thrown if the specified file does not exist.
    optional: bool,

    /// Whether the include was automatically generated
    ///
    /// Defaults to `false`. If `true`, then Stencila will automatically remove it, if based on
    /// file changes and dependency analysis, it is no longer needed.
    autogen: bool,
}

/// YAML syntax for `Include`
///
/// Allows for string or object.
#[skip_serializing_none]
#[derive(Deserialize, Serialize)]
#[serde(untagged, crate = "common::serde")]
enum IncludeSyntax {
    String(String),
    Object {
        taskfile: String,

        #[serde(default)]
        dir: Option<String>,

        #[serde(default, skip_serializing_if = "is_false")]
        optional: bool,

        #[serde(default, skip_serializing_if = "is_false")]
        autogen: bool,
    },
}

impl From<IncludeSyntax> for Include {
    fn from(syntax: IncludeSyntax) -> Self {
        match syntax {
            IncludeSyntax::String(taskfile) => Include {
                taskfile,
                ..Default::default()
            },
            IncludeSyntax::Object {
                taskfile,
                dir,
                optional,
                autogen,
            } => Include {
                taskfile,
                dir,
                optional,
                autogen,
            },
        }
    }
}

impl From<Include> for IncludeSyntax {
    fn from(include: Include) -> Self {
        if include.dir.is_none() && !include.optional && !include.autogen {
            IncludeSyntax::String(include.taskfile)
        } else {
            let Include {
                taskfile,
                dir,
                optional,
                autogen,
            } = include;
            IncludeSyntax::Object {
                taskfile,
                dir,
                optional,
                autogen,
            }
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(untagged, crate = "common::serde")]
pub enum Variable {
    /// A static value that will be assigned to the variable.
    #[serde(deserialize_with = "deserialize_string_from_bool_or_number")]
    Static(String),

    /// A shell command whose output (STDOUT) will be assigned to the variable.
    Dynamic { sh: String },
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(from = "TaskSyntax", into = "TaskSyntax", crate = "common::serde")]
pub struct Task {
    /// A short description of the task
    pub desc: Option<String>,

    /// A longer description of the task
    summary: Option<String>,

    /// A list of files that this task is dependent upon
    ///
    /// Relevant for `checksum` and `timestamp` methods. Can be file paths or star globs.
    sources: Vec<String>,

    /// The directory which this task should run in
    dir: Option<String>,

    /// Method for determining the status of the task
    ///
    /// Available options: `checksum` (default), `timestamp` and none.
    /// Can be overridden on a task by task basis.
    /// See https://taskfile.dev/usage/#prevent-unnecessary-work.
    method: Option<String>,

    /// Do not print task execution lines
    ///
    /// Defaults to `false`. See https://taskfile.dev/usage/#silent-mode.
    silent: bool,

    /// Whether the task should be run again or not if called more than once
    ///
    /// Defaults to global value set in the Taskfile.
    run: Option<String>,

    /// A prefix to print before `stdout`
    ///
    /// Only applicable when using the `prefixed` output mode.
    prefix: Option<String>,

    /// Continue execution if errors happen while executing the commands
    ignore_error: bool,

    /// Whether the task is automatically generated
    ///
    /// Defaults to `false`. If `true`, then Stencila will automatically update the
    /// task (including potentially removing it) based on file changes and dependency analysis.
    pub autogen: bool,

    /// A list of files that this task is generates
    ///
    /// Relevant for `timestamp` methods. Can be file paths or star globs.
    generates: Vec<String>,

    /// A list of commands to check if this task should run
    ///
    /// The task is skipped otherwise. This overrides `method`, `sources` and `generates`.
    status: Vec<String>,

    /// A list of commands to check if this task should run.
    preconditions: Vec<Precondition>,

    /// Task variables
    vars: IndexMap<String, Variable>,

    /// Task environment variables
    env: IndexMap<String, Variable>,

    /// A list of dependencies of this task
    deps: Vec<Dependency>,

    /// A list of commands to be executed for this task
    cmds: Vec<Command>,
}

impl Task {
    /// Is this task simple (only has `cmds`)?
    fn is_simple(&self) -> bool {
        self.desc.is_none()
            && self.summary.is_none()
            && !self.autogen
            && self.sources.is_empty()
            && self.dir.is_none()
            && self.method.is_none()
            && !self.silent
            && self.run.is_none()
            && self.prefix.is_none()
            && !self.ignore_error
            && self.generates.is_empty()
            && self.status.is_empty()
            && self.preconditions.is_empty()
            && self.vars.is_empty()
            && self.env.is_empty()
            && self.deps.is_empty()
    }
}

/// YAML syntax for `Task`
///
/// Allows for string, vector of strings, or object
#[allow(clippy::large_enum_variant)]
#[skip_serializing_none]
#[derive(Deserialize, Serialize)]
#[serde(untagged, crate = "common::serde")]
enum TaskSyntax {
    String(String),
    Vector(Vec<Command>),
    Object {
        #[serde(default)]
        desc: Option<String>,

        #[serde(default)]
        summary: Option<String>,

        #[serde(default, skip_serializing_if = "is_false")]
        autogen: bool,

        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        sources: Vec<String>,

        #[serde(default)]
        dir: Option<String>,

        #[serde(default)]
        method: Option<String>,

        #[serde(default, skip_serializing_if = "is_false")]
        silent: bool,

        #[serde(default)]
        run: Option<String>,

        #[serde(default)]
        prefix: Option<String>,

        #[serde(default, skip_serializing_if = "is_false")]
        ignore_error: bool,

        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        generates: Vec<String>,

        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        status: Vec<String>,

        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        preconditions: Vec<Precondition>,

        #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
        vars: IndexMap<String, Variable>,

        #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
        env: IndexMap<String, Variable>,

        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        deps: Vec<Dependency>,

        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        cmds: Vec<Command>,
    },
}

impl From<TaskSyntax> for Task {
    fn from(other: TaskSyntax) -> Self {
        match other {
            TaskSyntax::String(cmd) => Task {
                cmds: vec![Command {
                    cmd: Some(cmd),
                    ..Default::default()
                }],
                ..Default::default()
            },
            TaskSyntax::Vector(cmds) => Task {
                cmds,
                ..Default::default()
            },
            TaskSyntax::Object {
                desc,
                summary,
                autogen,
                sources,
                dir,
                method,
                silent,
                run,
                prefix,
                ignore_error,
                generates,
                status,
                preconditions,
                vars,
                env,
                deps,
                cmds,
            } => Task {
                desc,
                summary,
                autogen,
                sources,
                dir,
                method,
                silent,
                run,
                prefix,
                ignore_error,
                generates,
                status,
                preconditions,
                vars,
                env,
                deps,
                cmds,
            },
        }
    }
}

impl From<Task> for TaskSyntax {
    fn from(task: Task) -> Self {
        if task.is_simple() {
            if task.cmds.len() == 1 && task.cmds[0].is_simple() {
                if let Some(cmd) = task.cmds[0].cmd.clone() {
                    return TaskSyntax::String(cmd);
                }
            } else {
                return TaskSyntax::Vector(task.cmds);
            };
        }
        let Task {
            desc,
            summary,
            autogen,
            sources,
            dir,
            method,
            silent,
            run,
            prefix,
            ignore_error,
            generates,
            status,
            preconditions,
            vars,
            env,
            deps,
            cmds,
        } = task;
        TaskSyntax::Object {
            desc,
            summary,
            autogen,
            sources,
            dir,
            method,
            silent,
            run,
            prefix,
            ignore_error,
            generates,
            status,
            preconditions,
            vars,
            env,
            deps,
            cmds,
        }
    }
}

#[derive(Clone, Defaults, Deserialize, Serialize)]
#[serde(
    from = "PreconditionSyntax",
    into = "PreconditionSyntax",
    crate = "common::serde"
)]
pub struct Precondition {
    /// Command to be executed
    ///
    /// If a non-zero exit code is returned, the task errors without executing its commands.
    sh: String,

    /// Optional message to print if the precondition isn't met.
    msg: Option<String>,
}

/// YAML syntax for `Precondition`
///
/// Allows for string or object.
#[skip_serializing_none]
#[derive(Deserialize, Serialize)]
#[serde(untagged, crate = "common::serde")]
enum PreconditionSyntax {
    String(String),
    Object {
        sh: String,

        #[serde(default)]
        msg: Option<String>,
    },
}

impl From<PreconditionSyntax> for Precondition {
    fn from(syntax: PreconditionSyntax) -> Self {
        match syntax {
            PreconditionSyntax::String(sh) => Precondition {
                sh,
                ..Default::default()
            },
            PreconditionSyntax::Object { sh, msg } => Precondition { sh, msg },
        }
    }
}

impl From<Precondition> for PreconditionSyntax {
    fn from(precondition: Precondition) -> Self {
        if precondition.msg.is_none() {
            PreconditionSyntax::String(precondition.sh)
        } else {
            let Precondition { sh, msg } = precondition;
            PreconditionSyntax::Object { sh, msg }
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Defaults, Deserialize, Serialize)]
#[serde(
    from = "DependencySyntax",
    into = "DependencySyntax",
    crate = "common::serde"
)]
pub struct Dependency {
    /// The task to be executes as a dependency
    task: String,

    /// Optional additional variables to be passed to the referenced task
    vars: IndexMap<String, Variable>,
}

/// YAML syntax for `Dependency`
///
/// Allows for string or object.
#[skip_serializing_none]
#[derive(Deserialize, Serialize)]
#[serde(untagged, crate = "common::serde")]
enum DependencySyntax {
    String(String),
    Object {
        task: String,

        #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
        vars: IndexMap<String, Variable>,
    },
}

impl From<DependencySyntax> for Dependency {
    fn from(syntax: DependencySyntax) -> Self {
        match syntax {
            DependencySyntax::String(task) => Dependency {
                task,
                ..Default::default()
            },
            DependencySyntax::Object { task, vars } => Dependency { task, vars },
        }
    }
}

impl From<Dependency> for DependencySyntax {
    fn from(precondition: Dependency) -> Self {
        if precondition.vars.is_empty() {
            DependencySyntax::String(precondition.task)
        } else {
            let Dependency { task, vars } = precondition;
            DependencySyntax::Object { task, vars }
        }
    }
}

#[derive(Clone, Defaults, Deserialize, Serialize)]
#[serde(
    default,
    from = "CommandSyntax",
    into = "CommandSyntax",
    crate = "common::serde"
)]
pub struct Command {
    /// The shell command to be executed
    ///
    /// Should be `None` if `defer` or `task` are set.
    cmd: Option<String>,

    /// Schedules the command to be executed at the end of this task instead of immediately
    ///
    /// Cannot be used together with `cmd`.
    defer: Option<String>,

    /// Whether to display task runs
    ///
    /// Defaults to `false`. Overrides the `silent` option in the root of the Taskfile.
    silent: bool,

    /// Whether to display task runs
    ///
    /// Continue execution if errors happen while executing the command.
    ignore_error: bool,

    /// Set this to trigger execution of another task instead of running a command.
    ///
    /// This cannot be set together with cmd.
    task: Option<String>,

    /// Optional additional variables to be passed to the referenced task
    ///
    /// Only relevant when setting `task` instead of `cmd`.
    vars: IndexMap<String, Variable>,
}

impl Command {
    /// Is this command simple (only has `cmd`)?
    fn is_simple(&self) -> bool {
        self.cmd.is_some()
            && self.defer.is_none()
            && !self.silent
            && !self.ignore_error
            && self.task.is_none()
            && self.vars.is_empty()
    }
}

/// YAML syntax for `Command`
///
/// Allows for string, vector of strings, or object
#[allow(clippy::large_enum_variant)]
#[skip_serializing_none]
#[derive(Deserialize, Serialize)]
#[serde(untagged, crate = "common::serde")]
enum CommandSyntax {
    String(String),
    Object {
        #[serde(default)]
        cmd: Option<String>,

        #[serde(default)]
        defer: Option<String>,

        #[serde(default, skip_serializing_if = "is_false")]
        silent: bool,

        #[serde(default, skip_serializing_if = "is_false")]
        ignore_error: bool,

        #[serde(default)]
        task: Option<String>,

        #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
        vars: IndexMap<String, Variable>,
    },
}

impl From<CommandSyntax> for Command {
    fn from(other: CommandSyntax) -> Self {
        match other {
            CommandSyntax::String(cmd) => Command {
                cmd: Some(cmd),
                ..Default::default()
            },
            CommandSyntax::Object {
                cmd,
                defer,
                silent,
                ignore_error,
                task,
                vars,
            } => Command {
                cmd,
                defer,
                silent,
                ignore_error,
                task,
                vars,
            },
        }
    }
}

impl From<Command> for CommandSyntax {
    fn from(command: Command) -> Self {
        match command.is_simple() {
            true => CommandSyntax::String(command.cmd.expect("Simple command should have cmd")),
            false => {
                let Command {
                    cmd,
                    defer,
                    silent,
                    ignore_error,
                    task,
                    vars,
                } = command;
                CommandSyntax::Object {
                    cmd,
                    defer,
                    silent,
                    ignore_error,
                    task,
                    vars,
                }
            }
        }
    }
}

fn is_false(value: &bool) -> bool {
    !value
}

fn deserialize_string_from_bool_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged, crate = "common::serde")]
    enum AnyOf {
        Boolean(bool),
        Number(i64),
        Float(f64),
        String(String),
    }

    match AnyOf::deserialize(deserializer)? {
        AnyOf::Boolean(b) => Ok(b.to_string()),
        AnyOf::Number(i) => Ok(i.to_string()),
        AnyOf::Float(f) => Ok(f.to_string()),
        AnyOf::String(s) => Ok(s),
    }
}

#[derive(RustEmbed)]
#[folder = "taskfiles"]
struct Taskfiles;

#[cfg(test)]
mod test {
    use super::*;
    use test_snaps::{insta::assert_snapshot, snapshot_fixtures_content};

    #[test]
    fn serialization() {
        snapshot_fixtures_content("taskfiles/*.yaml", |yaml| {
            let taskfile = Taskfile::load(yaml).expect("Unable to load Taskfile");
            let yaml = taskfile.dump().expect("Unable to dump Taskfile");
            assert_snapshot!(yaml);
        });
    }
}
