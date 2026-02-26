//! CLI for managing agent definitions.
//!
//! Provides `stencila agents` subcommands: list, show, validate, create.

use std::{
    path::{Path, PathBuf},
    process::exit,
};

use clap::{Args, Parser, Subcommand};
use eyre::Result;

use stencila_cli_utils::{
    AsFormat, Code, ToStdout,
    color_print::cstr,
    message,
    tabulated::{Attribute, Cell, Color, Tabulated},
};
use stencila_codecs::{DecodeOptions, EncodeOptions, Format};
use stencila_schema::{Node, NodeType};

use crate::{
    agent_def::{self, AgentSource},
    agent_validate,
    convenience::{CreateAgentOptions, create_agent, create_session},
    types::{EventKind, SessionConfig},
};

/// Manage agent definitions
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

enum ResolvedValidationTarget {
    Path {
        path: PathBuf,
        dir_name: Option<String>,
    },
    Agent {
        agent: stencila_schema::Agent,
        dir_name: Option<String>,
    },
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all agents</dim>
  <b>stencila agents</>

  <dim># Show details about a specific agent</dim>
  <b>stencila agents show</> <g>code-review</>

  <dim># Validate an agent by name, directory, or file path</dim>
  <b>stencila agents validate</> <g>code-review</>

  <dim># Create a new agent in the workspace</dim>
  <b>stencila agents create</> <g>my-agent</> <y>\"A helpful assistant\"</>

  <dim># Create a new agent in user config</dim>
  <b>stencila agents create</> <g>my-agent</> <y>\"A helpful assistant\"</> <c>--user</>

  <dim># Run an agent with a prompt</dim>
  <b>stencila agents run</> <g>code-engineer</> <y>\"What files are in this directory?\"</>

  <dim># Dry run to see agent config and prompt</dim>
  <b>stencila agents run</> <g>code-engineer</> <y>\"Hello\"</> <c>--dry-run</>
"
);

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
    Show(Show),
    Validate(Validate),
    Create(Create),
    Run(Run),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            List::default().run().await?;
            return Ok(());
        };

        match command {
            Command::List(list) => list.run().await?,
            Command::Show(show) => show.run().await?,
            Command::Validate(validate) => validate.run().await?,
            Command::Create(create) => create.run().await?,
            Command::Run(run) => run.run().await?,
        }

        Ok(())
    }
}

/// List available agents
///
/// Shows agents from workspace `.stencila/agents/`, user config
/// `~/.config/stencila/agents/`, and auto-detected CLI tools on PATH.
/// Use `--source` to filter by source.
#[derive(Default, Debug, Args)]
#[command(after_long_help = LIST_AFTER_LONG_HELP)]
struct List {
    /// Output the list as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,

    /// Filter by source (may be repeated)
    #[arg(long, short, value_enum)]
    source: Vec<AgentSource>,
}

pub static LIST_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all agents in table format</dim>
  <b>stencila agents list</>

  <dim># Output agents as JSON</dim>
  <b>stencila agents list</> <c>--as</> <g>json</>

  <dim># List only workspace agents</dim>
  <b>stencila agents list</> <c>--source</> <g>workspace</>

  <dim># List only CLI-detected agents</dim>
  <b>stencila agents list</> <c>--source</> <g>cli</>
"
);

impl List {
    async fn run(self) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let all = agent_def::discover(&cwd).await;

        let list: Vec<_> = if self.source.is_empty() {
            all
        } else {
            all.into_iter()
                .filter(|a| a.source().is_some_and(|s| self.source.contains(&s)))
                .collect()
        };

        if let Some(format) = self.r#as {
            Code::new_from(format.into(), &list)?.to_stdout();
            return Ok(());
        }

        let mut table = Tabulated::new();
        table.set_header(["Name", "Description", "Source", "Model"]);

        for agent in list {
            let source_cell = match agent.source() {
                Some(AgentSource::Workspace) => Cell::new("workspace").fg(Color::Blue),
                Some(AgentSource::User) => Cell::new("user").fg(Color::Green),
                Some(AgentSource::CliDetected) => Cell::new("cli").fg(Color::Cyan),
                None => Cell::new("-").fg(Color::DarkGrey),
            };

            let model = agent.model.as_deref().unwrap_or("-");

            table.add_row([
                Cell::new(&agent.name).add_attribute(Attribute::Bold),
                Cell::new(&agent.description),
                source_cell,
                Cell::new(model),
            ]);
        }

        table.to_stdout();

        Ok(())
    }
}

/// Create a new agent
///
/// Creates a new agent directory with a template AGENT.md. By default creates
/// in the workspace's `.stencila/agents/` directory. Use `--user` to create in
/// `~/.config/stencila/agents/` instead.
#[derive(Debug, Args)]
#[command(after_long_help = CREATE_AFTER_LONG_HELP)]
struct Create {
    /// The name for the new agent
    ///
    /// Must be lowercase kebab-case: 1-64 characters, only lowercase alphanumeric
    /// and hyphens, no leading/trailing/consecutive hyphens. By convention, names
    /// follow a `thing-role` pattern describing the agent's domain and function,
    /// e.g. `code-engineer`, `code-reviewer`, `data-analyst`, `site-designer`.
    name: String,

    /// A brief description of the new agent
    description: String,

    /// Create in user config directory instead of workspace
    #[arg(long)]
    user: bool,
}

pub static CREATE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Create a new agent in the workspace</dim>
  <b>stencila agents create</> <g>my-agent</> <y>\"A helpful assistant\"</>

  <dim># Create a new agent in user config</dim>
  <b>stencila agents create</> <g>my-agent</> <y>\"A helpful assistant\"</> <c>--user</>
"
);

impl Create {
    async fn run(self) -> Result<()> {
        let options = CreateAgentOptions {
            user: self.user,
            ..Default::default()
        };
        let agent = create_agent(&self.name, &self.description, &options).await?;

        message!(
            "✨ Created agent `{}` at `{}`",
            agent.name,
            agent.home().display()
        );

        Ok(())
    }
}

/// Show an agent
///
/// Displays the full content and metadata of a specific agent.
#[derive(Debug, Args)]
#[command(after_long_help = SHOW_AFTER_LONG_HELP)]
struct Show {
    /// The name of the agent to show
    name: String,

    /// The format to show the agent in
    #[arg(long, short, default_value = "md")]
    r#as: Format,
}

pub static SHOW_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Show an agent as Markdown</dim>
  <b>stencila agents show</> <g>code-review</>

  <dim># Show an agent as JSON</dim>
  <b>stencila agents show</> <g>code-review</> <c>--as</> <g>json</>
"
);

impl Show {
    async fn run(self) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let agent = agent_def::get_by_name(&cwd, &self.name).await?;

        let content = stencila_codecs::to_string(
            &Node::Agent(agent.inner),
            Some(EncodeOptions {
                format: Some(self.r#as.clone()),
                ..Default::default()
            }),
        )
        .await?;

        Code::new(self.r#as, &content).to_stdout();

        Ok(())
    }
}

/// Validate an agent
///
/// Checks that an agent conforms to naming and property constraint rules.
/// Accepts an agent name, a directory path, or a path to an AGENT.md file.
#[derive(Debug, Args)]
#[command(after_long_help = VALIDATE_AFTER_LONG_HELP)]
struct Validate {
    /// Agent name, directory path, or AGENT.md path
    target: String,
}

pub static VALIDATE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Validate an agent by name</dim>
  <b>stencila agents validate</> <g>code-review</>

  <dim># Validate an agent directory</dim>
  <b>stencila agents validate</> <g>.stencila/agents/code-review</>

  <dim># Validate an AGENT.md file directly</dim>
  <b>stencila agents validate</> <g>.stencila/agents/code-review/AGENT.md</>
"
);

impl Validate {
    /// Resolve the target to either an AGENT.md path or an in-memory agent,
    /// plus optional directory name for name-vs-directory validation.
    async fn resolve_target(&self) -> Result<ResolvedValidationTarget> {
        let path = PathBuf::from(&self.target);

        // If target is a path to an AGENT.md file
        if path.is_file()
            && path
                .file_name()
                .is_some_and(|n| n.eq_ignore_ascii_case("AGENT.md"))
        {
            let dir_name = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(String::from);
            return Ok(ResolvedValidationTarget::Path { path, dir_name });
        }

        // If target is a directory (containing AGENT.md)
        if path.is_dir() {
            let agent_md = path.join("AGENT.md");
            if agent_md.exists() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).map(String::from);
                return Ok(ResolvedValidationTarget::Path {
                    path: agent_md,
                    dir_name,
                });
            }
            eyre::bail!("No AGENT.md found in directory `{}`", path.display());
        }

        // Otherwise, treat as an agent name — look up across all sources
        let cwd = std::env::current_dir()?;
        let agent = agent_def::get_by_name(&cwd, &self.target).await?;

        // CLI-detected agents are in-memory and have no AGENT.md path.
        let dir_name = if agent.path().as_os_str().is_empty() {
            None
        } else {
            agent
                .path()
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(String::from)
        };

        Ok(ResolvedValidationTarget::Agent {
            agent: agent.inner,
            dir_name,
        })
    }

    async fn parse_agent(path: &Path) -> Result<stencila_schema::Agent> {
        let content = tokio::fs::read_to_string(path).await?;
        let node = stencila_codecs::from_str(
            &content,
            Some(DecodeOptions {
                format: Some(Format::Markdown),
                node_type: Some(NodeType::Agent),
                ..Default::default()
            }),
        )
        .await?;

        let Node::Agent(agent) = node else {
            eyre::bail!("Failed to parse `{}` as an Agent", path.display());
        };

        Ok(agent)
    }

    async fn run(self) -> Result<()> {
        let (agent, dir_name) = match self.resolve_target().await? {
            ResolvedValidationTarget::Path { path, dir_name } => {
                let agent = Self::parse_agent(&path).await?;
                (agent, dir_name)
            }
            ResolvedValidationTarget::Agent { agent, dir_name } => (agent, dir_name),
        };

        let errors = agent_validate::validate_agent(&agent, dir_name.as_deref());

        if errors.is_empty() {
            message!("🎉 Agent `{}` is valid", agent.name);
            Ok(())
        } else {
            message!(
                "⚠️  Agent `{}` has {} error{}:",
                agent.name,
                errors.len(),
                if errors.len() > 1 { "s" } else { "" }
            );
            for error in &errors {
                message!("  - {}", error);
            }
            exit(1)
        }
    }
}

/// Run an agent with a prompt
///
/// Discovers a named agent definition, creates an agent session using the
/// agent's configuration (model, provider, instructions, tool settings), and
/// streams the response. Arguments that correspond to existing file paths are
/// read and included as file content. Mainly for testing.
#[derive(Debug, Args)]
#[command(after_long_help = RUN_AFTER_LONG_HELP)]
struct Run {
    /// The name of the agent to run
    name: String,

    /// Text prompts and/or file paths (automatically detected)
    args: Vec<String>,

    /// Write output to the specified file instead of stdout
    #[arg(long, short)]
    output: Option<PathBuf>,

    /// Show agent config and prompt without executing
    #[arg(long)]
    dry_run: bool,
}

pub static RUN_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Run an agent with a prompt</dim>
  <b>stencila agents run</> <g>code-engineer</> <y>\"What files are in this directory?\"</>

  <dim># Mix text and file paths</dim>
  <b>stencila agents run</> <g>code-review</> <y>\"Review this file:\"</> <g>src/main.rs</>

  <dim># Write output to a file</dim>
  <b>stencila agents run</> <g>code-engineer</> <y>\"Generate a README\"</> <c>--output</> <g>README.md</>

  <dim># Dry run to see agent config and prompt</dim>
  <b>stencila agents run</> <g>code-engineer</> <y>\"Hello\"</> <c>--dry-run</>
"
);

impl Run {
    #[allow(clippy::print_stdout, clippy::print_stderr)]
    async fn run(self) -> Result<()> {
        // Build prompt from args: detect file paths and read their content
        let prompt = build_prompt_from_args(&self.args)?;

        if prompt.is_empty() {
            return Err(eyre::eyre!(
                "No prompt provided. Pass text and/or file paths as arguments."
            ));
        }

        // Dry run: resolve the agent and show config without creating a session
        if self.dry_run {
            let cwd = std::env::current_dir()?;
            let agent = agent_def::get_by_name(&cwd, &self.name).await?;
            let config = SessionConfig::from_agent(&agent).await?;

            Code::new(Format::Markdown, "# Agent\n").to_stdout();
            Code::new(
                Format::Yaml,
                &format!("name: {}\ndescription: {}\n", agent.name, agent.description),
            )
            .to_stdout();

            Code::new(Format::Markdown, "\n# Prompt\n").to_stdout();
            Code::new(Format::Markdown, &prompt).to_stdout();

            if let Some(ref instr) = config.user_instructions {
                Code::new(Format::Markdown, "\n# Instructions\n").to_stdout();
                Code::new(Format::Markdown, instr).to_stdout();
            }

            Code::new(Format::Markdown, "\n# Session Config\n").to_stdout();
            Code::new_from(Format::Yaml, &config)?.to_stdout();

            return Ok(());
        }

        // Create session from agent definition and submit
        let (_agent, mut session, mut event_rx) = create_session(&self.name).await?;

        let mut submit_fut = Box::pin(session.submit(&prompt));
        let mut submit_done = false;
        let mut submit_result: Option<Result<(), crate::error::AgentError>> = None;
        let mut collected_text = String::new();
        let writing_to_file = self.output.is_some();

        loop {
            tokio::select! {
                biased;

                event = event_rx.recv() => {
                    let Some(event) = event else {
                        break;
                    };

                    match event.kind {
                        EventKind::AssistantTextDelta => {
                            if let Some(serde_json::Value::String(delta)) = event.data.get("delta") {
                                if !writing_to_file {
                                    print!("{delta}");
                                }
                                collected_text.push_str(delta);
                            }
                        }
                        EventKind::ToolCallStart => {
                            if let Some(serde_json::Value::String(tool_name)) = event.data.get("tool_name") {
                                let args = event.data.get("arguments").cloned().unwrap_or(serde_json::Value::Null);
                                eprintln!("[tool] {tool_name} {}", format_tool_args(&args));
                            }
                        }
                        EventKind::ToolCallEnd => {
                            if let Some(error) = event.data.get("error").and_then(serde_json::Value::as_str) {
                                let call_id = event.data.get("call_id").and_then(serde_json::Value::as_str).unwrap_or("?");
                                eprintln!("[tool error] {call_id}: {error}");
                            }
                        }
                        EventKind::TurnLimit => {
                            eprintln!("[warning] Turn limit reached");
                        }
                        EventKind::LoopDetection => {
                            let msg = event.data.get("message").and_then(serde_json::Value::as_str).unwrap_or("Loop detected");
                            eprintln!("[warning] {msg}");
                        }
                        EventKind::Info => {
                            let msg = event.data.get("message").and_then(serde_json::Value::as_str).unwrap_or("info");
                            eprintln!("[info] {msg}");
                        }
                        EventKind::Error => {
                            let msg = event.data.get("message").and_then(serde_json::Value::as_str).unwrap_or("unknown error");
                            eprintln!("[error] {msg}");
                        }
                        _ => {}
                    }
                }

                result = &mut submit_fut, if !submit_done => {
                    submit_done = true;
                    submit_result = Some(result);
                }
            }

            if submit_done {
                // Drain remaining buffered events
                while let Ok(event) = event_rx.try_recv() {
                    if let EventKind::AssistantTextDelta = event.kind
                        && let Some(serde_json::Value::String(delta)) = event.data.get("delta")
                    {
                        if !writing_to_file {
                            print!("{delta}");
                        }
                        collected_text.push_str(delta);
                    }
                }

                break;
            }
        }

        // Check for submit error
        if let Some(Err(e)) = submit_result {
            return Err(eyre::eyre!("Agent run failed: {e}"));
        }

        if let Some(ref path) = self.output {
            std::fs::write(path, &collected_text)
                .map_err(|e| eyre::eyre!("Failed to write {}: {e}", path.display()))?;
            message!("Wrote {} bytes to {}", collected_text.len(), path.display());
        } else if !collected_text.ends_with('\n') {
            println!();
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Build a prompt string from CLI arguments, reading file content for paths
/// that exist on disk.
fn build_prompt_from_args(args: &[String]) -> Result<String> {
    let mut parts = Vec::new();
    for arg in args {
        let path = PathBuf::from(arg);
        if path.exists() && path.is_file() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| eyre::eyre!("Failed to read {}: {e}", path.display()))?;
            parts.push(format!("--- {} ---\n{content}", path.display()));
        } else {
            parts.push(arg.clone());
        }
    }
    Ok(parts.join("\n\n"))
}

/// Format tool call arguments as a compact string for stderr display.
fn format_tool_args(arguments: &serde_json::Value) -> String {
    match arguments.as_object() {
        Some(obj) if !obj.is_empty() => {
            // Show first string-valued argument compactly
            for key in &["file_path", "path", "command", "pattern", "query", "name"] {
                if let Some(serde_json::Value::String(v)) = obj.get(*key) {
                    let display = if v.len() > 60 { &v[..60] } else { v };
                    return display.to_string();
                }
            }
            // Fallback: first string value
            for v in obj.values() {
                if let Some(s) = v.as_str() {
                    let display = if s.len() > 60 { &s[..60] } else { s };
                    return display.to_string();
                }
            }
            String::new()
        }
        _ => String::new(),
    }
}
