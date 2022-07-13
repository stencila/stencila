use std::{
    env::current_dir,
    fs::{create_dir_all, read_to_string, remove_file, write, File},
    path::{Path, PathBuf},
    time::Duration,
};

use fs2::FileExt;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use rust_embed::RustEmbed;

use binary_task::{BinaryTrait, TaskBinary};
use common::{
    defaults::Defaults,
    eyre::{bail, eyre, Result},
    futures::future,
    glob,
    indexmap::IndexMap,
    itertools::Itertools,
    once_cell::sync::Lazy,
    regex::Regex,
    serde::{Deserialize, Deserializer, Serialize},
    serde_with::skip_serializing_none,
    serde_yaml,
    slug::slugify,
    tokio::{self, sync::mpsc::error::TrySendError},
    tracing,
};

#[skip_serializing_none]
#[derive(Defaults, Deserialize, Serialize)]
#[serde(default, crate = "common::serde")]
pub struct Taskfile {
    /// The version of the Taskfile schema
    #[def = "\"3\".to_string()"]
    pub version: String,

    /// A short description of the taskfile
    pub desc: Option<String>,

    /// A longer description of the taskfile
    summary: Option<String>,

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

    /// The path of the Taskfile
    #[serde(skip)]
    path: PathBuf,
}

impl Taskfile {
    /// Create a new Taskfile at a path
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            ..Default::default()
        }
    }

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

    /// Get the path of the Taskfile
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    /// Get the directory of the Taskfile
    pub fn dir(&self) -> &Path {
        self.path
            .parent()
            .expect("Taskfiles should always have a parent directory")
    }

    /// Load a Taskfile from a YAML string
    fn load(yaml: &str) -> Result<Self> {
        let taskfile = serde_yaml::from_str(yaml)?;
        Ok(taskfile)
    }

    /// Read a Taskfile from a filesystem path
    ///
    /// If `inclusion_depth` is greater than zero, will add tasks from `includes` to the Taskfile's `tasks`.
    pub fn read(path: &Path, inclusion_depth: usize) -> Result<Self> {
        let path = Self::resolve(Some(path))?;
        if !path.exists() {
            bail!(
                "Could not find taskfile at `{}`; perhaps create one using `stencila tasks init`",
                path.display()
            )
        }

        let yaml = read_to_string(&path)?;

        let mut taskfile = match Self::load(&yaml) {
            Ok(taskfile) => taskfile,
            Err(error) => bail!("While reading Taskfile `{}`: {}", path.display(), error),
        };
        taskfile.path = path;

        if inclusion_depth == 0 {
            return Ok(taskfile);
        }

        let dir = taskfile.dir().to_path_buf();
        for (include_name, include) in &taskfile.includes {
            let include_path = dir.join(&include.taskfile);
            let include_path = Self::resolve(Some(&include_path))?;
            if !include.optional && !include_path.exists() {
                bail!(
                    "Included Taskfile does not exist: {}",
                    include_path.display()
                )
            };

            let include_taskfile = Self::read(&include_path, inclusion_depth - 1)?;
            for (task_name, task) in include_taskfile.tasks {
                taskfile
                    .tasks
                    .insert([include_name, ":", &task_name].concat(), task);
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
        let mut yaml = yaml
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

        if !yaml.ends_with('\n') {
            yaml.push('\n');
        }

        Ok(yaml)
    }

    /// Write the Taskfile to a filesystem path
    pub fn write(&self) -> Result<()> {
        let yaml = self.dump()?;
        write(self.path(), yaml)?;
        Ok(())
    }

    /// Initialize tasks in a directory
    ///
    /// Creates a `Taskfile.yaml` if one does not yet exist, and a `.stencila/tasks/lib` if one does not
    /// yet exist.
    pub async fn init(path: Option<&Path>, inclusion_depth: usize) -> Result<Taskfile> {
        let path = Self::resolve(path)?;

        // Copy all of the embedded taskfiles to .stencila/tasks in the directory
        // We do this, rather than use some other location, so it is easier to commit the taskfiles
        // or bundle up the whole project without breaking running of tasks.
        // Do it before reading the Taskfile to avoid error if included file is missing.
        let dir = path.parent().expect("Taskfile should always have a parent");
        let lib = dir.join(".stencila").join("tasks").join("lib");
        if !lib.exists() {
            create_dir_all(&lib)?;
            for name in Taskfiles::iter() {
                let name = name.to_string();
                if let Some(file) = Taskfiles::get(&name) {
                    let path = lib.join(&name);
                    write(&path, file.data)?;
                }
            }
        }

        let taskfile = match path.exists() {
            true => Taskfile::read(&path, inclusion_depth)?,
            false => {
                let mut taskfile = Taskfile::new(path);

                // Initialize the taskfile by doing a detect on in.
                taskfile.detect(true).await?;

                // Write the taskfile and then re-read it so that `inclusion_depth` is respected
                // e.g. when running tasks for a newly created taskfile
                taskfile.write()?;
                Taskfile::read(taskfile.path(), inclusion_depth)?
            }
        };

        Ok(taskfile)
    }

    /// Detect the tasks needed for a directory
    ///
    /// Updates the Taskfile in the directory (creating one if necessary) with tasks detected
    /// by running the `lib:detect` task.
    pub async fn detect(&mut self, run_task: bool) -> Result<()> {
        // Remove any existing tasks `autogen` includes and tasks
        // If this is not done they may accumulate even if not needed
        self.includes.retain(|_name, include| !include.autogen);
        self.tasks.retain(|_name, task| !task.autogen);

        // Ensure that there is an include entry for the library
        // Previously we also added individual includes for the lib modules e;.g. `python`, `asdf`.
        // This avoids having to append `lib:` when running tasks.
        // However, this pollutes the Taskfile and is sensitive to the order of includes - for some
        // reason `task` crashes when a yaml that includes another yaml is included before the first.
        if self.includes.get("lib").is_none() {
            self.includes.insert(
                "lib".to_string(),
                Include {
                    taskfile: ".stencila/tasks/lib".to_string(),
                    autogen: true,
                    ..Default::default()
                },
            );
        }

        // Add a `detect` command if necessary
        if !self.tasks.contains_key("detect") {
            self
                .tasks
                .insert("detect".to_string(), Task {
                    desc: Some("Detect which tasks are required".to_string()),
                    summary: Some(
                        "Autogenerated task which analyzes the files in the project and determines which tasks are required to build it."
                            .to_string(),
                    ),
                    autogen: true,
                    cmds: vec![
                        Command::task("lib:detect")
                    ],
                    ..Default::default()
                });
        }

        // Run the Taskfile's detect task
        if run_task {
            self.write()?;
            Task::run_now(self.path(), "detect", Vec::new()).await?;
        }

        // Read in the detected tasks
        let detected = self.dir().join(".stencila").join("tasks").join("detected");
        let tasks = read_to_string(&detected).unwrap_or_default();

        // Group detected tasks into "meta" tasks
        let mut pull_cmds = Vec::new();
        let mut install_cmds = Vec::new();
        for task in tasks.lines() {
            if let Some((_namespace, action)) = task.splitn(2, ':').collect_tuple() {
                if action == "pull" {
                    pull_cmds.push(Command::task(&["lib:", task].concat()))
                } else if action == "install" {
                    install_cmds.push(Command::task(&["lib:", task].concat()))
                }
            }
        }

        let mut build_cmds = Vec::new();

        // Add a `pull` command if necessary
        if !pull_cmds.is_empty() && !self.tasks.contains_key("install") {
            self.tasks.insert(
                "pull".to_string(),
                Task {
                    desc: Some("Pull the project and its sources".to_string()),
                    summary: Some(
                        "Autogenerated task which pulls the project, and any sources, from providers."
                            .to_string(),
                    ),
                    autogen: true,
                    cmds: pull_cmds,
                    ..Default::default()
                },
            );
            build_cmds.push(Command::task("pull"))
        }

        // If there is a detect task, all it to the build commands, after `pull`
        if self.tasks.contains_key("detect") {
            build_cmds.push(Command::task("detect"))
        }

        // Add an `install` command if necessary
        if !install_cmds.is_empty() && !self.tasks.contains_key("install") {
            self.tasks.insert(
                "install".to_string(),
                Task {
                    desc: Some("Install tools and packages".to_string()),
                    summary: Some(
                        "Autogenerated task which runs all install tasks that were detected for the project."
                            .to_string(),
                    ),
                    autogen: true,
                    cmds: install_cmds,
                    ..Default::default()
                },
            );
            build_cmds.push(Command::task("install"))
        }

        let mut run_cmds = Vec::new();

        // Add a `build` command if necessary
        if !build_cmds.is_empty() && !self.tasks.contains_key("build") {
            self.tasks.insert(
                "build".to_string(),
                Task {
                    desc: Some("Build the project".to_string()),
                    summary: Some(
                        "Autogenerated task which runs all build tasks that were detected for the project.".to_string(),
                    ),
                    autogen: true,
                    cmds: build_cmds,
                    ..Default::default()
                },
            );
            run_cmds.push(Command::task("build"))
        }

        self.write()
    }

    /// Run one or more tasks in the Taskfile
    ///
    /// Reads the Taskfile, including included tasks, to be able to check that the
    /// desired task is available before spawning `task`. If not present, then attempts
    /// to resolve it by pre-pending `lib:` to the name.
    pub async fn run(
        &self,
        args: &Vec<String>,
        now: bool,
        schedule: Option<&str>,
        watch: Option<&str>,
        ignore: Option<&str>,
        delay: Option<u64>,
    ) -> Result<()> {
        if args.is_empty() {
            bail!("No arguments provided to `task run`")
        }

        // Parse the args into name of task and its vars
        let mut task: String = String::new();
        let mut next_task: String = String::new();
        let mut vars: Vec<String> = Vec::new();
        for index in 0..(args.len() + 1) {
            let arg = args.get(index);
            if let Some(arg) = arg {
                if let Some((name, value)) = arg.splitn(2, '=').collect_tuple() {
                    let var = format!("{}={}", name.to_uppercase(), value);
                    vars.push(var);
                } else if task.is_empty() {
                    task = arg.to_string();
                } else {
                    next_task = arg.to_string();
                }
            }

            if !next_task.is_empty() || index == args.len() {
                let name = if !self.tasks.contains_key(&task) {
                    let lib_task = ["lib:", &task].concat();
                    let lib_util_task = ["lib:util:", &task].concat();
                    if self.tasks.contains_key(&lib_task) {
                        lib_task
                    } else if self.tasks.contains_key(&lib_util_task) {
                        lib_util_task
                    } else {
                        bail!(
                            "Taskfile does not have task named `{}`, `{}`, or `{}`",
                            task,
                            lib_task,
                            lib_util_task
                        )
                    }
                } else {
                    task.to_string()
                };

                if let Some(task) = self.tasks.get(&name) {
                    task.run(
                        self.path(),
                        &name,
                        vars.clone(),
                        now,
                        schedule,
                        watch,
                        ignore,
                        delay,
                    )
                    .await?;
                } else {
                    // This should never be reached!
                    bail!("Taskfile does not have task named `{}`", name);
                }

                task = next_task.clone();
                next_task = String::new();
                vars.clear();
            }
        }

        Ok(())
    }

    /// Generated Markdown documentation for the Taskfile
    fn docs(&self, name: &str) -> Result<String> {
        let title = format!(
            "# `{}`: {}",
            name,
            self.desc
                .clone()
                .unwrap_or_else(|| format!("Tasks related to `{}`", name))
        );

        let summary = self.summary.clone().unwrap_or_default();

        let includes = if self.includes.is_empty() {
            String::new()
        } else {
            format!(
                "## Includes\n\nOther `Taskfile`s included:\n\n{}",
                self.includes
                    .keys()
                    .map(|other| format!("- [`{}`]({}.md)\n", other, other))
                    .collect_vec()
                    .concat()
            )
        };

        let template_vars = if self.vars.is_empty() {
            String::new()
        } else {
            format!(
                "## Template variables\n\n{}",
                self.vars
                    .iter()
                    .map(|(name, var)| var.docs(name))
                    .collect_vec()
                    .concat()
            )
        };

        let environment_vars = if self.env.is_empty() {
            String::new()
        } else {
            format!(
                "## Environment variables\n\n{}",
                self.env
                    .iter()
                    .map(|(name, var)| var.docs(name))
                    .collect_vec()
                    .concat()
            )
        };

        let tasks = self
            .tasks
            .iter()
            .filter(|(_name, task)| !task.hide)
            .collect_vec();
        let tasks = if tasks.is_empty() {
            String::new()
        } else {
            format!(
                "## Tasks\n\n{}",
                self.tasks
                    .iter()
                    .map(|(name, task)| task.docs(name))
                    .collect_vec()
                    .concat()
            )
        };

        let md = format!(
            r"
<!-- Generated from Taskfile. Do not edit. -->

{title}

{summary}
{includes}
{template_vars}
{environment_vars}
{tasks}"
        );

        Ok(md)
    }

    /// Generate Markdown documentation for all Taskfiles in the library
    pub fn docs_all() -> Result<()> {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("reference")
            .join("tasks");
        create_dir_all(&dir)?;

        let taskfiles = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("taskfiles")
            .read_dir()?
            .flatten()
            .filter(|entry| {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                entry.path().is_file() && name.ends_with(".yaml") && name != "Taskfile.yaml"
            })
            .sorted_by(|a, b| a.file_name().cmp(&b.file_name()));

        let mut table = "| Topic | Description |\n| --- | --- |\n".to_string();
        for taskfile in taskfiles {
            let path = taskfile.path();
            let name = path
                .file_stem()
                .ok_or_else(|| eyre!("File has no stem"))?
                .to_string_lossy();

            let taskfile = Taskfile::read(&path, 0)?;
            let md = taskfile.docs(&name)?;
            let path = dir.join(format!("{}.md", name));
            write(path, md.trim())?;

            table += &format!(
                "| [`{}`]({}.md) | {} |\n",
                name,
                name,
                taskfile.desc.unwrap_or_default()
            );
        }

        let readme = dir.join("README.md");
        let mut content = read_to_string(&readme)?;
        content.replace_range(
            (content.find("<!-- TASKS-START -->").unwrap_or_default() + 21)
                ..content.find("<!-- TASKS-FINISH -->").unwrap_or_default(),
            &table,
        );
        write(readme, content)?;

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

impl Variable {
    fn docs(&self, name: &str) -> String {
        format!(
            "- `{}`: {}\n",
            name,
            match self {
                Variable::Static(value) => format!("`{}`", value),
                Variable::Dynamic { sh } => format!("`{}` (dynamic)", sh),
            }
        )
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(from = "TaskSyntax", into = "TaskSyntax", crate = "common::serde")]
pub struct Task {
    /// A short description of the task
    pub desc: Option<String>,

    /// A longer description of the task
    summary: Option<String>,

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

    /// Whether the task should be hidden from task lists
    ///
    /// Defaults to `false`. Used to hide helper tasks e.g. OS-specific tasks
    /// from lists.
    pub hide: bool,

    /// Whether the task is automatically generated
    ///
    /// Defaults to `false`. If `true`, then Stencila will automatically update the
    /// task (including potentially removing it) based on file changes and dependency analysis.
    pub autogen: bool,

    /// A schedule for running the task
    schedule: Vec<Schedule>,

    /// A list of files that this task watches for changes
    ///
    /// Can be file paths or star globs. Use an empty list if the task has sources
    /// but you do not want these to be watched
    watches: Vec<Watch>,

    /// A list of files that this task is dependent upon
    ///
    /// Relevant for `checksum` and `timestamp` methods. Can be file paths or star globs.
    sources: Vec<String>,

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
            && self.dir.is_none()
            && self.method.is_none()
            && !self.silent
            && self.run.is_none()
            && self.prefix.is_none()
            && !self.ignore_error
            && self.schedule.is_empty()
            && self.watches.is_empty()
            && self.sources.is_empty()
            && self.generates.is_empty()
            && self.status.is_empty()
            && self.preconditions.is_empty()
            && self.vars.is_empty()
            && self.env.is_empty()
            && self.deps.is_empty()
    }

    /// Run this task
    #[allow(clippy::too_many_arguments)]
    pub async fn run(
        &self,
        path: &Path,
        name: &str,
        vars: Vec<String>,
        now: bool,
        schedule: Option<&str>,
        watch: Option<&str>,
        ignore: Option<&str>,
        delay: Option<u64>,
    ) -> Result<()> {
        let dir = path
            .parent()
            .expect("Taskfile should always have parent directory");

        if now && schedule.is_some() {
            tracing::warn!("Ignoring `schedule` because `now` option used")
        }
        if now && watch.is_some() {
            tracing::warn!("Ignoring `watch` because `now` option used")
        }

        let schedules = schedule
            .map(|when| {
                vec![Schedule {
                    when: when.to_string(),
                    ..Default::default()
                }]
            })
            .unwrap_or_else(|| self.schedule.clone());

        let watches = watch
            .map(|files| {
                vec![Watch {
                    pattern: files.to_string(),
                    ignore: ignore.map(String::from),
                    delay,
                }]
            })
            .unwrap_or_else(|| self.watches.clone());

        if now || (schedules.is_empty() && watches.is_empty()) {
            return Task::run_now(path, name, vars).await;
        }

        // Start a thread that will run the task when notified by a schedule or by a watch
        let (run_sender, mut run_receiver) = tokio::sync::mpsc::channel(1);
        let path_clone = path.to_path_buf();
        let name_clone = name.to_string();
        tokio::spawn(async move {
            while let Some(..) = run_receiver.recv().await {
                tracing::debug!("Received run event for task `{}`", name_clone);
                if let Err(error) = Task::run_now(&path_clone, &name_clone, vars.clone()).await {
                    tracing::error!("While running task {}: {}", name_clone, error)
                }
            }
        });

        let mut handles = Vec::new();

        // Run each of the schedules asynchronously
        for schedule in schedules {
            let run_sender_clone = run_sender.clone();
            let handle = tokio::spawn(async move {
                let Schedule { when, tz } = schedule;
                if let Err(error) = cron_utils::run(&when, tz, run_sender_clone).await {
                    tracing::error!("While running schedule `{}`: {}", when, error)
                }
            });
            handles.push(handle);
        }

        // Run file watches
        for watch in watches {
            // Determine the path that should be watched from the glob by taking chars up to first glob special character
            let watch_path = watch
                .pattern
                .chars()
                .take_while(|&char| char != '*' && char != '?' && char != '[' && char != ']')
                .collect::<String>();
            let watch_path = dir.join(watch_path);

            if !watch_path.exists() {
                bail!("Path to watch does not exist: {}", watch_path.display());
            }

            let watch_pattern = match glob::Pattern::new(&watch.pattern) {
                Ok(pattern) => pattern,
                Err(error) => bail!("While parsing watch pattern: {}", error),
            };

            let mut ignores = watch.ignore.map(|ignore| vec![ignore]).unwrap_or_default();
            ignores.push(".stencila/tasks/**/*".to_string());

            let mut ignore_patterns = Vec::new();
            for ignore in ignores {
                let pattern = match glob::Pattern::new(&ignore) {
                    Ok(pattern) => pattern,
                    Err(error) => bail!("While parsing ignore pattern: {}", error),
                };
                ignore_patterns.push(pattern);
            }

            let (event_sender, mut event_receiver) = tokio::sync::mpsc::channel(10);

            // A standard synchronous thread for running watcher
            // This needs to be in a separate thread because `watch_receiver.recv()` is blocking an so prevents proper
            // functioning of `event_sender` async send (we use blocking send instead)
            let dir_clone = dir.to_path_buf();
            let ignores_clone = ignore_patterns.clone();
            std::thread::spawn(move || {
                let (watch_sender, watch_receiver) = std::sync::mpsc::channel();
                let mut watcher = watcher(watch_sender, Duration::from_millis(100)).unwrap();
                watcher.watch(watch_path, RecursiveMode::Recursive).unwrap();

                'events: loop {
                    let path = match watch_receiver.recv() {
                        Ok(event) => match event {
                            DebouncedEvent::Chmod(path)
                            | DebouncedEvent::Create(path)
                            | DebouncedEvent::Remove(path)
                            | DebouncedEvent::Write(path) => path,
                            _ => continue,
                        },
                        Err(error) => {
                            tracing::error!("While receiving watch event: {}", error);
                            continue;
                        }
                    };

                    tracing::debug!("Watch event: {}", path.display());
                    let path = match path.strip_prefix(&dir_clone) {
                        Ok(path) => path,
                        Err(error) => {
                            tracing::error!("While stripping prefix: {}", error);
                            continue;
                        }
                    };

                    if !watch_pattern.matches_path(path) {
                        tracing::debug!("Watch does not include path: {}", path.display());
                        continue;
                    }

                    for ignore in &ignores_clone {
                        if ignore.matches_path(path) {
                            tracing::debug!(
                                "Pattern `{}` ignores path: {}",
                                ignore,
                                path.display()
                            );
                            continue 'events;
                        }
                    }

                    if let Err(error) = event_sender.blocking_send(path.to_path_buf()) {
                        tracing::error!("While sending watch event: {}", error);
                    }
                }
            });

            // Async task to receive event paths and debounce them
            // The watcher thread debounces events for a particular file. This debounces for all watched files.
            let run_sender_clone = run_sender.clone();
            let delay = watch.delay.unwrap_or(0);
            let duration = Duration::from_secs(delay);
            let handle = tokio::spawn(async move {
                let mut events = false;
                loop {
                    match tokio::time::timeout(duration, event_receiver.recv()).await {
                        Ok(Some(..)) => {
                            // Watch event happened
                            events = true;
                        }
                        Ok(None) => {
                            // Event receiver dropped
                            break;
                        }
                        Err(..) => {
                            // Timeout happened so trigger task if events since last here
                            if events {
                                tracing::info!("Sending watch event");
                                if let Err(error) = run_sender_clone.try_send(()) {
                                    match error {
                                        TrySendError::Full(..) => tracing::debug!(
                                            "Task is running and a re-run is already queued"
                                        ),
                                        TrySendError::Closed(..) => break,
                                    }
                                }
                                events = false;
                            }
                        }
                    };
                }
            });
            handles.push(handle);
        }

        future::join_all(handles).await;

        Ok(())
    }

    pub async fn run_now(path: &Path, name: &str, mut vars: Vec<String>) -> Result<()> {
        tracing::debug!("Running task `{}` of `{}`", name, path.display());

        let mut binary = TaskBinary {}.ensure().await?;
        binary.env_list(&[("TASK_TEMP_DIR", "./.stencila/tasks")]);

        let mut task_args = vec![format!("--taskfile={}", path.display()), name.to_string()];
        task_args.append(&mut vars);

        let locks = path
            .parent()
            .expect("Should have parent")
            .join(".stencila")
            .join("tasks")
            .join("locks");
        create_dir_all(&locks)?;

        let lock_path = locks.join(slugify(name));
        let lock = File::create(&lock_path)?;
        if let Err(..) = lock.try_lock_exclusive() {
            tracing::info!("Task is already running, skipping a re-run");
            return Ok(());
        }

        let result = binary
            .run_with(
                task_args,
                Some(tracing::Level::INFO),
                Some(tracing::Level::INFO),
            )
            .await;

        lock.unlock()?;
        remove_file(lock_path)?;

        result
    }

    /// Generate Markdown documentation
    fn docs(&self, name: &str) -> String {
        let heading = format!(
            "### <a id='{}'>`{}`</a> : {}\n\n",
            name,
            name,
            self.desc.clone().unwrap_or_default()
        );

        let summary = self.summary.clone().unwrap_or_default();

        let sources = if self.sources.is_empty() {
            String::new()
        } else {
            format!(
                "#### Sources\n\n{}\n",
                self.sources
                    .iter()
                    .map(|source| format!("- `{}`", source))
                    .collect_vec()
                    .concat()
            )
        };

        let commands = if self.cmds.is_empty() {
            String::new()
        } else {
            format!(
                "#### Command{}\n\n{}",
                if self.cmds.len() == 1 { "" } else { "s" },
                self.cmds
                    .iter()
                    .enumerate()
                    .map(|(index, cmd)| cmd.docs(index, self.cmds.len()))
                    .collect_vec()
                    .concat()
            )
        };

        format!(
            r"
{heading}
{summary}
{sources}
{commands}
"
        )
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

        #[serde(default, skip_serializing_if = "is_false")]
        hide: bool,

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

        #[serde(
            default,
            alias = "schedules",
            skip_serializing_if = "OneOrMany::is_empty"
        )]
        schedule: OneOrMany<Schedule>,

        #[serde(default, alias = "watch", skip_serializing_if = "OneOrMany::is_empty")]
        watches: OneOrMany<Watch>,

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
                hide,
                sources,
                dir,
                method,
                silent,
                run,
                prefix,
                ignore_error,
                schedule,
                watches,
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
                hide,
                sources,
                dir,
                method,
                silent,
                run,
                prefix,
                ignore_error,
                schedule: schedule.into(),
                watches: watches.into(),
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
            hide,
            dir,
            method,
            silent,
            run,
            prefix,
            ignore_error,
            schedule,
            watches,
            sources,
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
            hide,
            dir,
            method,
            silent,
            run,
            prefix,
            ignore_error,
            schedule: schedule.into(),
            watches: watches.into(),
            sources,
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

/// YAML syntax for one or many of a type
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged, crate = "common::serde")]
enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    fn is_empty(&self) -> bool {
        match self {
            Self::One(..) => false,
            Self::Many(many) => many.is_empty(),
        }
    }
}

impl<T> Default for OneOrMany<T> {
    fn default() -> Self {
        Self::Many(Vec::new())
    }
}

impl<T> From<OneOrMany<T>> for Vec<T> {
    fn from(syntax: OneOrMany<T>) -> Self {
        match syntax {
            OneOrMany::One(one) => vec![one],
            OneOrMany::Many(many) => many,
        }
    }
}

impl<T> From<Vec<T>> for OneOrMany<T> {
    fn from(mut many: Vec<T>) -> Self {
        match many.len() == 1 {
            true => Self::One(many.remove(0)),
            false => Self::Many(many),
        }
    }
}

#[derive(Clone, Defaults, Deserialize, Serialize)]
#[serde(
    from = "ScheduleSyntax",
    into = "ScheduleSyntax",
    crate = "common::serde"
)]
pub struct Schedule {
    /// A cron expression or phrase
    when: String,

    /// Optional message to print if the precondition isn't met.
    tz: Option<String>,
}

/// YAML syntax for `Schedule`
///
/// Allows for string or object.
#[skip_serializing_none]
#[derive(Deserialize, Serialize)]
#[serde(untagged, crate = "common::serde")]
enum ScheduleSyntax {
    String(String),
    Object {
        when: String,

        #[serde(default)]
        tz: Option<String>,
    },
}

impl From<ScheduleSyntax> for Schedule {
    fn from(syntax: ScheduleSyntax) -> Self {
        match syntax {
            ScheduleSyntax::String(when) => Schedule {
                when,
                ..Default::default()
            },
            ScheduleSyntax::Object { when, tz } => Schedule { when, tz },
        }
    }
}

impl From<Schedule> for ScheduleSyntax {
    fn from(schedule: Schedule) -> Self {
        if schedule.tz.is_none() {
            ScheduleSyntax::String(schedule.when)
        } else {
            let Schedule { when, tz } = schedule;
            ScheduleSyntax::Object { when, tz }
        }
    }
}

#[derive(Clone, Defaults, Deserialize, Serialize)]
#[serde(
    from = "WatchesSyntax",
    into = "WatchesSyntax",
    crate = "common::serde"
)]
pub struct Watch {
    /// A cron expression or phrase
    pattern: String,

    /// Optional message to print if the precondition isn't met.
    ignore: Option<String>,

    /// Optional message to print if the precondition isn't met.
    delay: Option<u64>,
}

/// YAML syntax for `Watches`
///
/// Allows for string or object.
#[skip_serializing_none]
#[derive(Deserialize, Serialize)]
#[serde(untagged, crate = "common::serde")]
enum WatchesSyntax {
    String(String),
    Object {
        files: String,

        #[serde(default)]
        ignores: Option<String>,

        #[serde(default)]
        delay: Option<u64>,
    },
}

impl From<WatchesSyntax> for Watch {
    fn from(syntax: WatchesSyntax) -> Self {
        match syntax {
            WatchesSyntax::String(files) => Watch {
                pattern: files,
                ..Default::default()
            },
            WatchesSyntax::Object {
                files,
                ignores,
                delay,
            } => Watch {
                pattern: files,
                ignore: ignores,
                delay,
            },
        }
    }
}

impl From<Watch> for WatchesSyntax {
    fn from(watches: Watch) -> Self {
        if watches.ignore.is_none() && watches.delay.is_none() {
            WatchesSyntax::String(watches.pattern)
        } else {
            let Watch {
                pattern: files,
                ignore: ignores,
                delay,
            } = watches;
            WatchesSyntax::Object {
                files,
                ignores,
                delay,
            }
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
    /// Create a new command with only the `task` attribute
    fn task(task: &str) -> Self {
        Self {
            task: Some(task.to_string()),
            ..Default::default()
        }
    }

    /// Is this command simple (only has `cmd`)?
    fn is_simple(&self) -> bool {
        self.cmd.is_some()
            && self.defer.is_none()
            && !self.silent
            && !self.ignore_error
            && self.task.is_none()
            && self.vars.is_empty()
    }

    /// Generate Markdown documentation
    fn docs(&self, index: usize, of: usize) -> String {
        if let Some(cmd) = &self.cmd {
            if index == 0 && of == 1 {
                format!("```sh\n{}\n```\n\n", cmd)
            } else {
                let lines = cmd.split('\n').collect_vec();
                let code = if lines.len() > 1 {
                    [lines.first().expect(""), " ..."].concat()
                } else {
                    cmd.clone()
                };
                format!("{}. `{}`\n\n", index + 1, code)
            }
        } else if let Some(task) = &self.task {
            let link =
                if let Some((namespace, namespace_task)) = task.splitn(2, ':').collect_tuple() {
                    format!("[`{}`]({}.md#{})", task, namespace, namespace_task)
                } else {
                    format!("[`{}`](#{})", task, task)
                };
            let vars = self
                .vars
                .iter()
                .map(|(name, var)| match var {
                    Variable::Static(value) => format!(" `{}={}`", name, value),
                    Variable::Dynamic { sh } => format!("`{}=$({})`", name, sh),
                })
                .collect_vec()
                .concat();
            format!("{}. {} {}\n\n", index + 1, link, vars)
        } else {
            String::new()
        }
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
