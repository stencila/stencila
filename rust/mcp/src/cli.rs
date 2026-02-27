//! CLI subcommands for MCP server management.
//!
//! Provides `stencila mcp` with subcommands to list, show, add, and remove
//! MCP server configurations.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use eyre::Result;
use stencila_ask::{Answer, ask_with_default};
use stencila_cli_utils::{
    AsFormat, Code, ToStdout,
    color_print::cstr,
    message,
    tabulated::{Attribute, Cell, Color, Tabulated},
};

/// Manage MCP (Model Context Protocol) servers
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all discovered MCP servers</dim>
  <b>stencila mcp</>

  <dim># Show details about a specific server</dim>
  <b>stencila mcp show</> <g>filesystem</>

  <dim># Show a server's tools (connects to the server)</dim>
  <b>stencila mcp show</> <g>filesystem</> <c>--tools</>

  <dim># Add a stdio server</dim>
  <b>stencila mcp add</> <g>filesystem npx -y @modelcontextprotocol/server-filesystem /tmp</>

  <dim># Add an HTTP server</dim>
  <b>stencila mcp add</> <g>remote-api https://api.example.com/mcp</>

  <dim># Remove a server</dim>
  <b>stencila mcp remove</> <g>filesystem</>

  <dim># Print TypeScript declarations for all servers</dim>
  <b>stencila mcp codemode</>

  <dim># Print declarations for a specific server only</dim>
  <b>stencila mcp codemode</> <c>--server</> <g>filesystem</>
"
);

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
    Show(Show),
    Add(Add),
    Remove(Remove),
    Codemode(Codemode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Scope {
    User,
    Workspace,
}

impl Cli {
    /// Extract the [`Codemode`] args if the subcommand is `codemode`.
    ///
    /// Returns `Ok(args)` for the codemode subcommand, or `Err(self)` for
    /// everything else (giving ownership back so [`run`](Self::run) can be
    /// called). The caller (top-level CLI) uses this to handle codemode
    /// execution where `stencila-codemode` is available, avoiding a circular
    /// dependency.
    ///
    /// # Errors
    ///
    /// Returns `Err(self)` when the subcommand is not `codemode`.
    #[allow(clippy::result_large_err)]
    pub fn into_codemode(self) -> Result<Codemode, Self> {
        match self.command {
            Some(Command::Codemode(c)) => Ok(c),
            _ => Err(self),
        }
    }

    /// Run the MCP CLI command.
    ///
    /// # Errors
    ///
    /// Returns an error if the subcommand fails.
    pub async fn run(self) -> eyre::Result<()> {
        let Some(command) = self.command else {
            return List::default().run();
        };

        match command {
            Command::List(list) => list.run(),
            Command::Show(show) => show.run().await,
            Command::Add(add) => add.run().await,
            Command::Remove(remove) => remove.run(),
            Command::Codemode(_) => {
                unreachable!("codemode subcommand should be handled via Cli::into_codemode()")
            }
        }
    }
}

/// List all discovered MCP servers
///
/// Shows servers from all sources: Stencila config, Claude, Codex, and Gemini.
/// Servers are discovered from both user-level and workspace-level configs.
#[derive(Default, Debug, Args)]
#[command(after_long_help = LIST_AFTER_LONG_HELP)]
struct List {
    /// Output format (json or yaml)
    #[arg(long, short)]
    r#as: Option<AsFormat>,

    /// Workspace directory to discover servers from
    #[arg(long, default_value = ".")]
    dir: PathBuf,
}

pub static LIST_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all servers in table format</dim>
  <b>stencila mcp list</>

  <dim># Output as JSON</dim>
  <b>stencila mcp list</> <c>--as</> <g>json</>

  <dim># List servers for a specific workspace</dim>
  <b>stencila mcp list</> <c>--dir</> <g>./my-project</>
"
);

impl List {
    fn run(self) -> Result<()> {
        let dir = self.dir.canonicalize().unwrap_or(self.dir);
        let servers = crate::discover(&dir);

        if let Some(format) = self.r#as {
            Code::new_from(format.into(), &servers)?.to_stdout();
            return Ok(());
        }

        if servers.is_empty() {
            message!("No MCP servers discovered. Use `stencila mcp add` to add one.");
            return Ok(());
        }

        let mut table = Tabulated::new();
        table.set_header(["ID", "Source", "Enabled", "Transport"]);

        for server in &servers {
            let source_cell = match &server.source {
                Some(src) => {
                    let label = src.to_string();
                    match src {
                        crate::ConfigSource::Stencila => Cell::new(label).fg(Color::Blue),
                        crate::ConfigSource::ClaudeUser | crate::ConfigSource::ClaudeWorkspace => {
                            Cell::new(label).fg(Color::Green)
                        }
                        crate::ConfigSource::CodexUser | crate::ConfigSource::CodexWorkspace => {
                            Cell::new(label).fg(Color::Yellow)
                        }
                        crate::ConfigSource::GeminiUser | crate::ConfigSource::GeminiWorkspace => {
                            Cell::new(label).fg(Color::Cyan)
                        }
                    }
                }
                None => Cell::new("-").fg(Color::DarkGrey),
            };

            let enabled_cell = if server.enabled {
                Cell::new("yes").fg(Color::Green)
            } else {
                Cell::new("no").fg(Color::Red)
            };

            let transport_cell = format_transport(&server.transport);

            table.add_row([
                Cell::new(&server.id).add_attribute(Attribute::Bold),
                source_cell,
                enabled_cell,
                transport_cell,
            ]);
        }

        table.to_stdout();

        Ok(())
    }
}

/// Show details of an MCP server
///
/// Displays the configuration for a server discovered from any source.
/// Use `--tools` to connect to the server and list its available tools.
#[derive(Debug, Args)]
#[command(after_long_help = SHOW_AFTER_LONG_HELP)]
struct Show {
    /// The server ID to show
    id: String,

    /// Connect and list the server's tools
    #[arg(long)]
    tools: bool,

    /// Output format (json or yaml)
    #[arg(long, short)]
    r#as: Option<AsFormat>,

    /// Workspace directory
    #[arg(long, default_value = ".")]
    dir: PathBuf,
}

pub static SHOW_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Show server configuration</dim>
  <b>stencila mcp show</> <g>filesystem</>

  <dim># Show as JSON</dim>
  <b>stencila mcp show</> <g>filesystem</> <c>--as</> <g>json</>

  <dim># Connect and list tools</dim>
  <b>stencila mcp show</> <g>filesystem</> <c>--tools</>
"
);

impl Show {
    async fn run(self) -> Result<()> {
        let dir = self.dir.canonicalize().unwrap_or(self.dir);
        let servers = crate::discover(&dir);

        let server_config = servers
            .into_iter()
            .find(|s| s.id == self.id)
            .ok_or_else(|| eyre::eyre!("Server `{}` not found", self.id))?;

        if !self.tools {
            if let Some(format) = self.r#as {
                Code::new_from(format.into(), &server_config)?.to_stdout();
            } else {
                Code::new_from(
                    stencila_cli_utils::stencila_format::Format::Json,
                    &server_config,
                )?
                .to_stdout();
            }
            return Ok(());
        }

        // Connect and list tools
        message!("Connecting to `{}`...", self.id);

        let live = crate::LiveMcpServer::connect(server_config).await?;
        let tools = crate::McpServer::tools(&live).await?;

        if let Some(format) = self.r#as {
            Code::new_from(format.into(), &tools)?.to_stdout();
            return Ok(());
        }

        if tools.is_empty() {
            message!("Server `{}` has no tools.", self.id);
        } else {
            let mut table = Tabulated::new();
            table.set_header(["Tool", "Description"]);

            for tool in &tools {
                let desc = tool.description.as_deref().unwrap_or("-");
                table.add_row([
                    Cell::new(&tool.name).add_attribute(Attribute::Bold),
                    Cell::new(desc),
                ]);
            }

            table.to_stdout();
        }

        live.shutdown().await?;

        Ok(())
    }
}

/// Add an MCP server to stencila.toml
///
/// Adds a new server configuration to the nearest `stencila.toml` file,
/// or to the user-level config with `--user`.
///
/// The server spec is either a URL (for HTTP servers) or a command followed
/// by its arguments (for stdio servers). URLs are detected by an `http://`
/// or `https://` prefix.
#[derive(Debug, Args)]
#[command(after_long_help = ADD_AFTER_LONG_HELP)]
struct Add {
    /// The server ID (unique identifier)
    id: String,

    /// Command and arguments, or URL
    ///
    /// For stdio: the command followed by its arguments (e.g. `npx -y @pkg/name`)
    /// For HTTP: a URL starting with http:// or https://
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, required = true)]
    spec: Vec<String>,

    /// Human-readable name for the server
    #[arg(long)]
    name: Option<String>,

    /// Environment variable for stdio servers (repeatable, KEY=VALUE)
    #[arg(long, value_name = "KEY=VALUE")]
    env: Vec<String>,

    /// Overwrite if a server with this ID already exists
    #[arg(long, short)]
    force: bool,

    /// Add to user config (~/.config/stencila/stencila.toml)
    #[arg(long, conflicts_with_all = ["scope", "workspace"])]
    user: bool,

    /// Add to workspace config (nearest stencila.toml)
    #[arg(long, conflicts_with_all = ["scope", "user"])]
    workspace: bool,

    /// Config scope (compatibility with other tools)
    #[arg(long, value_enum, conflicts_with_all = ["user", "workspace"])]
    scope: Option<Scope>,

    /// Workspace directory
    #[arg(long, default_value = ".")]
    dir: PathBuf,
}

pub static ADD_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Add a stdio server</dim>
  <b>stencila mcp add</> <g>filesystem npx -y @modelcontextprotocol/server-filesystem /tmp</>

  <dim># Add an HTTP server</dim>
  <b>stencila mcp add</> <g>remote-api https://api.example.com/mcp</>

  <dim># Add with a display name</dim>
  <b>stencila mcp add</> <c>--name</> <g>'Filesystem Server'</> <g>fs npx -y @modelcontextprotocol/server-filesystem /tmp</>

  <dim># Add with environment variables</dim>
  <b>stencila mcp add</> <c>--env</> <g>GITHUB_TOKEN=ghp_xxx</> <g>github npx -y @modelcontextprotocol/server-github</>

  <dim># Add to user-level config</dim>
  <b>stencila mcp add</> <c>--user</> <g>my-server my-mcp-server</>

  <dim># Compatibility scope syntax</dim>
  <b>stencila mcp add</> <c>--scope</> <g>user</> <g>my-server my-mcp-server</>
"
);

impl Add {
    async fn run(self) -> Result<()> {
        // Check if a server with this ID already exists
        let dir = self.dir.canonicalize().unwrap_or(self.dir);
        let existing = crate::discover(&dir);
        if existing.iter().any(|s| s.id == self.id) && !self.force {
            let answer = ask_with_default(
                &format!("Server `{}` already exists. Overwrite?", self.id),
                Answer::No,
            )
            .await?;
            if !answer.is_yes() {
                message!("🚫 Add cancelled");
                return Ok(());
            }
        }

        let target = if self.user || matches!(self.scope, Some(Scope::User)) {
            stencila_config::ConfigTarget::User
        } else if self.workspace || matches!(self.scope, Some(Scope::Workspace)) {
            stencila_config::ConfigTarget::Nearest
        } else {
            // Default scope is workspace (nearest stencila.toml)
            stencila_config::ConfigTarget::Nearest
        };

        let prefix = format!("mcp.servers.{}", self.id);
        let first = &self.spec[0];

        if first.starts_with("http://") || first.starts_with("https://") {
            // HTTP transport — first arg is the URL
            stencila_config::set_value(&format!("{prefix}.transport.type"), "http", target)?;
            stencila_config::set_value(&format!("{prefix}.transport.url"), first, target)?;
        } else {
            // Stdio transport — first arg is command, rest are args
            stencila_config::set_value(&format!("{prefix}.transport.type"), "stdio", target)?;
            stencila_config::set_value(&format!("{prefix}.transport.command"), first, target)?;

            let args = &self.spec[1..];
            if !args.is_empty() {
                let args_value = format!(
                    "[{}]",
                    args.iter()
                        .map(|a| format!("\"{a}\""))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                stencila_config::set_value(
                    &format!("{prefix}.transport.args"),
                    &args_value,
                    target,
                )?;
            }
        }

        // Set environment variables
        for entry in &self.env {
            let (key, value) = entry
                .split_once('=')
                .ok_or_else(|| eyre::eyre!("Invalid --env value `{entry}`: expected KEY=VALUE"))?;
            stencila_config::set_value(&format!("{prefix}.env.{key}"), value, target)?;
        }

        // Set optional name
        if let Some(ref name) = self.name {
            stencila_config::set_value(&format!("{prefix}.name"), name, target)?;
        }

        let location = match target {
            stencila_config::ConfigTarget::User => "user config",
            stencila_config::ConfigTarget::Nearest | stencila_config::ConfigTarget::Local => {
                "stencila.toml"
            }
        };
        message!("Added MCP server `{}` to {}", self.id, location);

        Ok(())
    }
}

/// Remove an MCP server from stencila.toml
///
/// Removes a server configuration from the nearest `stencila.toml` file,
/// or from the user-level config with `--user`. Only Stencila-managed
/// servers can be removed (not those from Claude, Codex, or Gemini configs).
#[derive(Debug, Args)]
#[command(alias = "rm", after_long_help = REMOVE_AFTER_LONG_HELP)]
struct Remove {
    /// The server ID to remove
    id: String,

    /// Remove from user config (~/.config/stencila/stencila.toml)
    #[arg(long, conflicts_with_all = ["scope", "workspace"])]
    user: bool,

    /// Remove from workspace config (nearest stencila.toml)
    #[arg(long, conflicts_with_all = ["scope", "user"])]
    workspace: bool,

    /// Config scope (compatibility with other tools)
    #[arg(long, value_enum, conflicts_with_all = ["user", "workspace"])]
    scope: Option<Scope>,
}

pub static REMOVE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Remove from nearest stencila.toml</dim>
  <b>stencila mcp remove</> <g>filesystem</>

  <dim># Remove from user config</dim>
  <b>stencila mcp remove</> <c>--user</> <g>filesystem</>

  <dim># Compatibility scope syntax</dim>
  <b>stencila mcp remove</> <c>--scope</> <g>user</> <g>filesystem</>
"
);

impl Remove {
    fn run(self) -> Result<()> {
        let target = if self.user || matches!(self.scope, Some(Scope::User)) {
            stencila_config::ConfigTarget::User
        } else if self.workspace || matches!(self.scope, Some(Scope::Workspace)) {
            stencila_config::ConfigTarget::Nearest
        } else {
            // Default scope is workspace (nearest stencila.toml)
            stencila_config::ConfigTarget::Nearest
        };

        let key = format!("mcp.servers.{}", self.id);
        let config_file = match stencila_config::unset_value(&key, target) {
            Ok(config_file) => config_file,
            Err(error) => {
                let error_message = error.to_string();
                if error_message.contains(&format!("Key path not found: {key}"))
                    || error_message.contains(&format!("Key not found: {key}"))
                {
                    return Err(eyre::eyre!("No mcp server found with id `{}`", self.id));
                }

                return Err(error);
            }
        };

        message!(
            "Removed MCP server `{}` from `{}`",
            self.id,
            config_file.display()
        );

        Ok(())
    }
}

/// Print generated TypeScript declarations for MCP codemode
///
/// Connects to discovered MCP servers, fetches their tools, and prints the
/// `.d.ts` declarations that would be injected at runtime into the codemode
/// sandbox. Useful for verifying what an LLM agent sees.
#[derive(Debug, Args)]
#[command(after_long_help = CODEMODE_AFTER_LONG_HELP)]
pub struct Codemode {
    /// Only include specific server(s) (repeatable)
    #[arg(long, value_name = "ID")]
    pub server: Vec<String>,

    /// Workspace directory to discover servers from
    #[arg(long, default_value = ".")]
    pub dir: PathBuf,
}

pub static CODEMODE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Print declarations for all available servers</dim>
  <b>stencila mcp codemode</>

  <dim># Print declarations for a specific server</dim>
  <b>stencila mcp codemode</> <c>--server</> <g>filesystem</>

  <dim># Print declarations for multiple servers</dim>
  <b>stencila mcp codemode</> <c>--server</> <g>filesystem</> <c>--server</> <g>github</>

  <dim># Specify a workspace directory</dim>
  <b>stencila mcp codemode</> <c>--dir</> <g>./my-project</>
"
);

/// Format a transport config as a compact, colored [`Cell`] for table display.
fn format_transport(transport: &crate::TransportConfig) -> Cell {
    match transport {
        crate::TransportConfig::Stdio { command, args } => {
            let display = compact_command(command, args);
            let color = match command.as_str() {
                "npx" => Color::Cyan,
                "uvx" => Color::Magenta,
                "python" | "python3" => Color::Green,
                _ => Color::Yellow,
            };
            Cell::new(display).fg(color)
        }
        crate::TransportConfig::Http { url, .. } => Cell::new(url).fg(Color::Blue),
    }
}

/// Build a compact `command arg1 arg2 ...` string, truncating long arg lists.
fn compact_command(command: &str, args: &[String]) -> String {
    const MAX_ARGS: usize = 3;

    if args.is_empty() {
        return command.to_string();
    }

    if args.len() <= MAX_ARGS {
        format!("{command} {}", args.join(" "))
    } else {
        let shown: Vec<&str> = args.iter().take(MAX_ARGS).map(String::as_str).collect();
        format!("{command} {} ...", shown.join(" "))
    }
}
