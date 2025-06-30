use std::{path::PathBuf, process::exit};

use cli_utils::{
    table::{self, Attribute, Cell, CellAlignment, Color},
    AsFormat, Code, ToStdout,
};
use common::{
    clap::{self, Args, Parser, Subcommand},
    eyre::Result,
    itertools::Itertools,
    serde_json::json,
};

use crate::ToolType;

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
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            return List::default().run().await;
        };

        match command {
            Command::List(list) => list.run().await,
            Command::Show(show) => show.run().await,
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
            if self.available && !tool.path().is_some() {
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

        let home = directories::UserDirs::new().map(|dirs| dirs.home_dir().to_path_buf());

        for tool in list {
            let name = tool.name();
            let description = tool.description();
            let r#type = tool.r#type();
            let version_required = tool.version_required().to_string();
            let version_available = tool
                .version_available()
                .map_or_else(|| "-".into(), |version| version.to_string());
            let path = tool.path().map_or_else(
                || "-".into(),
                |path| {
                    if let Some(rest) = home.as_ref().and_then(|home| path.strip_prefix(home).ok())
                    {
                        PathBuf::from("~").join(rest)
                    } else {
                        path
                    }
                    .to_string_lossy()
                    .to_string()
                },
            );

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
            "Path": tool.path().map_or_else(|| "None".into(), |path| path.to_string_lossy().to_string()),
        });

        Code::new_from(AsFormat::Yaml.into(), &tool)?.to_stdout();

        Ok(())
    }
}
