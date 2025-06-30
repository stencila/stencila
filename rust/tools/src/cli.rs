use std::{
    fs::canonicalize,
    path::{Path, PathBuf},
    process::exit,
};

use cli_utils::{
    format::Format,
    table::{self, Attribute, Cell, CellAlignment, Color},
    AsFormat, Code, ToStdout,
};
use common::{
    clap::{self, Args, Parser, Subcommand},
    eyre::{bail, Result},
    itertools::Itertools,
    once_cell::sync::Lazy,
    serde_json::json,
};
use directories::UserDirs;

use crate::{detect_all_managers, ToolType};

/// Manage tools used by Stencila
#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
    Show(Show),
    Env(Env),
    Run(Run),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            return List::default().run().await;
        };

        match command {
            Command::List(list) => list.run().await,
            Command::Show(show) => show.run().await,
            Command::Env(detect) => detect.run().await,
            Command::Run(run) => run.run().await,
        }
    }
}

/// List tools used by Stencila
#[derive(Debug, Default, Args)]
struct List {
    /// Only list tools of a particular type
    #[arg(short, long)]
    r#type: Option<ToolType>,

    /// Only list tools that are available on the current machine
    #[arg(long)]
    available: bool,

    /// Output the tools as Model Context Protocol tool specifications in JSON or YAML
    ///
    /// See https://modelcontextprotocol.io/docs/concepts/tools for more details
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

impl List {
    async fn run(self) -> Result<()> {
        let list = super::list();

        let list = list.into_iter().filter(|tool| {
            if self.available && tool.path().is_none() {
                return false;
            }

            self.r#type
                .as_ref()
                .map(|tool_type| tool.r#type() == *tool_type)
                .unwrap_or(true)
        });

        if let Some(format) = self.r#as {
            let list = list.into_iter().map(|tool| tool.mcp_tool()).collect_vec();

            Code::new_from(format.into(), &list)?.to_stdout();

            return Ok(());
        }

        let mut table = table::new();
        table.set_header([
            "Name",
            "Description",
            "Type",
            "Ver. Required",
            "Ver. Available",
            "Path",
        ]);

        for tool in list {
            let name = tool.name();
            let description = tool.description();
            let r#type = tool.r#type();
            let version_required = tool.version_required().to_string();
            let version_available = tool
                .version_available()
                .map_or_else(|| "-".into(), |version| version.to_string());
            let path = tool
                .path()
                .map_or_else(|| "-".into(), |path| strip_home_dir(&path));

            table.add_row([
                Cell::new(name).add_attribute(Attribute::Bold),
                Cell::new(description),
                match r#type {
                    ToolType::Environments => Cell::new(r#type.to_string()).fg(Color::Magenta),
                    ToolType::Packages => Cell::new(r#type.to_string()).fg(Color::Blue),
                    ToolType::Execution => Cell::new(r#type.to_string()).fg(Color::Cyan),
                    ToolType::Linting => Cell::new(r#type.to_string()).fg(Color::Green),
                    ToolType::Conversion => Cell::new(r#type.to_string()).fg(Color::Yellow),
                    ToolType::Collaboration => Cell::new(r#type.to_string()).fg(Color::Red),
                },
                Cell::new(version_required).set_alignment(CellAlignment::Right),
                Cell::new(version_available).set_alignment(CellAlignment::Right),
                Cell::new(path),
            ]);
        }

        table.to_stdout();

        Ok(())
    }
}

/// Show details for a tool
#[derive(Debug, Default, Args)]
struct Show {
    /// The name of the tool to show details for
    name: String,
}

impl Show {
    #[allow(clippy::print_stderr)]
    async fn run(self) -> Result<()> {
        let Some(tool) = super::get(&self.name) else {
            eprintln!("🔍 No tool with name `{}`", self.name);
            exit(1)
        };

        let tool = json!({
            "Name": tool.name(),
            "URL": tool.url(),
            "Description": tool.description(),
            "Version required": tool.version_required(),
            "Version available": tool.version_available().map_or_else(|| "None".into(), |version| version.to_string()),
            "Path": tool.path().map_or_else(|| "None".into(), |path| strip_home_dir(&path)),
        });

        Code::new_from(Format::Yaml, &tool)?.to_stdout();

        Ok(())
    }
}

/// Detect environment manager configuration for a path
#[derive(Debug, Args)]
struct Env {
    /// The directory or file to check for environment manager configuration
    #[arg(default_value = ".")]
    path: PathBuf,
}

impl Env {
    #[allow(clippy::print_stderr)]
    async fn run(self) -> Result<()> {
        if !self.path.exists() {
            bail!("Path does not exist: {}", self.path.display());
        }

        let path = canonicalize(&self.path).unwrap_or(self.path);

        let managers = detect_all_managers(&path);

        if managers.is_empty() {
            eprintln!(
                "🔍 No environment or package manager configuration found for directory {}",
                strip_home_dir(&path)
            );
            exit(1)
        };

        for (tool, config_path) in detect_all_managers(&path) {
            let env = json!({
                "Tool": tool.name(),
                "Config file": strip_home_dir(&config_path)
            });
            Code::new_from(Format::Yaml, &env)?.to_stdout();

            let content = std::fs::read_to_string(&config_path)?;
            let format = Format::from_path(&config_path);
            Code::new(format, &content).to_stdout();
        }

        Ok(())
    }
}

/// Run a command through environment manager detection
#[derive(Debug, Args)]
struct Run {
    /// Working directory for the command
    #[arg(long, short = 'C')]
    cwd: Option<PathBuf>,

    /// The command and arguments to run (after --)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

impl Run {
    async fn run(self) -> Result<()> {
        if self.args.is_empty() {
            bail!("No command specified. Use -- to separate command from options, e.g.: stencila tools run -- pandoc --version");
        }

        let command = &self.args[0];
        let args = &self.args[1..];

        let mut cmd = crate::EnvironmentCommand::new(command);
        cmd.args(args);

        if let Some(cwd) = &self.cwd {
            if !cwd.exists() {
                bail!("Working directory does not exist: {}", cwd.display());
            }
            cmd.current_dir(cwd);
        }

        let status = cmd.status()?;
        if !status.success() {
            if let Some(code) = status.code() {
                exit(code);
            } else {
                exit(1);
            }
        }

        Ok(())
    }
}

/// Strip the home directory from a path and replace it with `~`
fn strip_home_dir(path: &Path) -> String {
    static HOME: Lazy<Option<PathBuf>> =
        Lazy::new(|| UserDirs::new().map(|dirs| dirs.home_dir().to_path_buf()));

    if let Some(rest) = HOME.as_ref().and_then(|home| path.strip_prefix(home).ok()) {
        PathBuf::from("~").join(rest)
    } else {
        path.to_path_buf()
    }
    .to_string_lossy()
    .to_string()
}
