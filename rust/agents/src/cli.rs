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

  <dim># Create a new agent in the workspace</dim>
  <b>stencila agents create</> <g>my-agent</> <y>\"A helpful assistant\"</>

  <dim># Create a new agent in user config</dim>
  <b>stencila agents create</> <g>my-agent</> <y>\"A helpful assistant\"</> <c>--user</>

  <dim># Validate an agent by name, directory, or file path</dim>
  <b>stencila agents validate</> <g>code-review</>

  <dim># Show how an agent session would be routed</dim>
  <b>stencila agents resolve</> <g>code-engineer</>

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
    Create(Create),
    Validate(Validate),
    Resolve(Resolve),
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
            Command::Create(create) => create.run().await?,
            Command::Validate(validate) => validate.run().await?,
            Command::Resolve(resolve) => resolve.run().await?,
            Command::Run(run) => run.run().await?,
        }

        Ok(())
    }
}

fn summarize_agent_details(agent: &stencila_schema::Agent) -> Option<String> {
    let mut details = Vec::new();

    if let Some(model) = agent.model.as_deref()
        && !model.is_empty()
    {
        details.push(format!("model={model}"));
    }

    if let Some(provider) = agent.provider.as_deref()
        && !provider.is_empty()
    {
        details.push(format!("provider={provider}"));
    }

    if let Some(reasoning) = agent.reasoning_effort.as_deref()
        && !reasoning.is_empty()
    {
        details.push(format!("reasoning={reasoning}"));
    }

    if let Some(skills) = agent.allowed_skills.as_deref()
        && !skills.is_empty()
    {
        details.push(format!("skills={}", summarize_string_list(skills)));
    }

    if let Some(tools) = agent.allowed_tools.as_deref()
        && !tools.is_empty()
    {
        details.push(format!("tools={}", summarize_string_list(tools)));
    }

    if let Some(enable_mcp) = agent.options.enable_mcp {
        details.push(format!("mcp={}", if enable_mcp { "on" } else { "off" }));
    }

    if let Some(enable_mcp_codemode) = agent.options.enable_mcp_codemode {
        details.push(format!(
            "mcp-codemode={}",
            if enable_mcp_codemode { "on" } else { "off" }
        ));
    }

    if let Some(servers) = agent.options.allowed_mcp_servers.as_deref()
        && !servers.is_empty()
    {
        details.push(format!("mcp-servers={}", summarize_string_list(servers)));
    }

    if let Some(max_turns) = agent.options.max_turns {
        details.push(format!("max-turns={max_turns}"));
    }

    if let Some(tool_timeout) = agent.options.tool_timeout {
        details.push(format!("tool-timeout={tool_timeout}s"));
    }

    if let Some(max_tool_rounds) = agent.options.max_tool_rounds {
        details.push(format!("max-tool-rounds={max_tool_rounds}"));
    }

    if let Some(max_subagent_depth) = agent.options.max_subagent_depth {
        details.push(format!("max-subagent-depth={max_subagent_depth}"));
    }

    if let Some(compatibility) = agent.options.compatibility.as_deref()
        && !compatibility.is_empty()
    {
        details.push(format!(
            "compatibility={}",
            summarize_text(compatibility, 40)
        ));
    }

    if details.is_empty() {
        None
    } else {
        Some(details.join("; "))
    }
}

fn summarize_string_list(items: &[String]) -> String {
    const MAX_ITEMS: usize = 3;

    let shown = items
        .iter()
        .take(MAX_ITEMS)
        .map(String::as_str)
        .collect::<Vec<_>>();
    let remaining = items.len().saturating_sub(MAX_ITEMS);

    if remaining == 0 {
        shown.join(",")
    } else {
        format!("{},+{remaining}", shown.join(","))
    }
}

fn summarize_text(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    let count = trimmed.chars().count();

    if count <= max_chars {
        return trimmed.to_string();
    }

    let prefix = trimmed.chars().take(max_chars).collect::<String>();
    format!("{prefix}…")
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
        table.set_header(["Name", "Description", "Source", "Details"]);

        for agent in list {
            let source_cell = match agent.source() {
                Some(AgentSource::Workspace) => Cell::new("workspace").fg(Color::Blue),
                Some(AgentSource::User) => Cell::new("user").fg(Color::Green),
                Some(AgentSource::CliDetected) => Cell::new("cli").fg(Color::Cyan),
                None => Cell::new("-").fg(Color::DarkGrey),
            };

            let details = summarize_agent_details(&agent.inner)
                .map(Cell::new)
                .unwrap_or_else(|| Cell::new("-").fg(Color::DarkGrey));

            table.add_row([
                Cell::new(&agent.name).add_attribute(Attribute::Bold),
                Cell::new(&agent.description),
                source_cell,
                details,
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

/// Show how an agent session would be routed
///
/// Dry-run routing resolution: shows the provider, model, session type,
/// credential source, and reasoning without starting a session. Useful for
/// debugging why a particular provider or model was chosen.
#[derive(Debug, Args)]
#[command(after_long_help = RESOLVE_AFTER_LONG_HELP)]
struct Resolve {
    /// The name of the agent to resolve
    name: String,

    /// Show extended routing details
    #[arg(long)]
    why: bool,
}

pub static RESOLVE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Show routing for the default agent</dim>
  <b>stencila agents resolve</> <g>default</>

  <dim># Show extended routing details</dim>
  <b>stencila agents resolve</> <g>code-engineer</> <c>--why</>
"
);

impl Resolve {
    #[allow(clippy::print_stdout)]
    async fn run(self) -> Result<()> {
        use crate::routing;

        let resolved_name = crate::convenience::resolve_default_agent_name(&self.name).await;
        let cwd = std::env::current_dir()?;
        let agent = agent_def::get_by_name(&cwd, &resolved_name).await?;

        let client = stencila_models3::client::Client::from_env().ok();
        let no_api_client = client.is_none();
        let empty_client;
        let client_ref = match client.as_ref() {
            Some(c) => c,
            None => {
                empty_client = stencila_models3::client::Client::builder()
                    .build()
                    .map_err(|e| eyre::eyre!("{e}"))?;
                &empty_client
            }
        };

        if no_api_client {
            eprintln!("Note: no API client configured; showing CLI fallback route");
        }

        let decision = routing::route_session_explained(
            agent.provider.as_deref(),
            agent.model.as_deref(),
            client_ref,
        )
        .map_err(|e| eyre::eyre!("{e}"))?;

        let (backend, provider, model) = match &decision.route {
            routing::SessionRoute::Api { provider, model } => {
                ("API", provider.as_str(), Some(model.as_str()))
            }
            routing::SessionRoute::Cli { provider, model } => {
                ("CLI", provider.as_str(), model.as_deref())
            }
        };

        let model_display = if let Some((alias, concrete)) = &decision.alias_resolution {
            format!("{alias} → {concrete}")
        } else if let Some(m) = model {
            m.to_string()
        } else {
            "default".to_string()
        };

        let credential_display = client_ref
            .get_credential_source(provider)
            .map_or_else(|| "-".to_string(), |s| s.to_string());

        println!("Agent:       {} ({})", agent.name, agent.home().display());
        println!("Provider:    {} ({})", provider, decision.provider_source);
        println!("Model:       {model_display}");
        println!("Session:     {backend}");
        println!("Credentials: {credential_display}");

        if decision.fallback_used {
            let reason = decision
                .fallback_reason
                .as_deref()
                .unwrap_or("no API credentials");
            println!("Fallback:    ⚠️  {reason}");
        }

        if self.why {
            println!();
            println!("Extended Details\n");
            println!("Provider source:  {}", decision.provider_source);
            println!("Model source:     {}", decision.model_source);

            let mut provider_names = client_ref.provider_names();
            provider_names.sort();
            if provider_names.is_empty() {
                println!("API providers:    (none configured)");
            } else {
                println!("API providers:    {}", provider_names.join(", "));
            }

            if let Some(configured) = stencila_config::load_and_validate(&cwd)
                .ok()
                .and_then(|c| c.models)
                .and_then(|m| m.providers)
            {
                println!("Config priority:  {}", configured.join(", "));
            } else {
                println!("Config priority:  (not set; using registration order)");
            }

            for name in &provider_names {
                if let Some(source) = client_ref.get_credential_source(name) {
                    println!("  {name}: {source}");
                }
            }
        }

        Ok(())
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
                        EventKind::Warning => {
                            let msg = event.data.get("message").and_then(serde_json::Value::as_str).unwrap_or("warning");
                            eprintln!("[warning] {msg}");
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
