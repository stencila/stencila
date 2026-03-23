//! CLI for managing agent definitions.
//!
//! Provides `stencila agents` subcommands: list, show, validate, create.

use std::{
    path::{Path, PathBuf},
    process::exit,
    sync::Arc,
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
    convenience::{
        CreateAgentOptions, CreateSessionOptions, create_agent, create_session_with_options,
    },
    definition::{self, AgentSource},
    migrations::AGENT_MIGRATIONS,
    store::{AgentSessionStore, ListSessionsFilter, SessionPersistence, SessionRecord},
    types::{EventKind, SessionConfig, Turn},
    validate,
};

/// Resolve the configured global provider preference list from `stencila.toml`.
///
/// This helper is duplicated in the CLI module, rather than reusing the
/// similarly named function in `convenience.rs`, because `agents resolve`
/// needs to load configuration relative to the specific current working
/// directory already resolved for this command.
///
/// The `convenience.rs` helper uses `stencila_config::get()`, which is fine
/// for normal session creation, but the CLI resolution path already has the
/// command-local `cwd` in hand and should use `load_and_validate(cwd)` so the
/// displayed resolution result is derived from the same workspace context that
/// the command is inspecting. Keeping this helper local also avoids widening
/// visibility of an otherwise internal convenience-layer function just for CLI
/// reporting.
fn resolve_configured_model_providers(cwd: &Path) -> Option<Vec<String>> {
    stencila_config::load_and_validate(cwd)
        .ok()
        .and_then(|config| config.models)
        .and_then(|models| models.providers)
}

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
        agent_path: PathBuf,
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
    Sessions(Sessions),
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
            Command::Sessions(sessions) => sessions.run().await?,
        }

        Ok(())
    }
}

fn summarize_agent_details(agent: &stencila_schema::Agent) -> Option<String> {
    let mut details = Vec::new();

    if let Some(models) = agent.models.as_deref()
        && !models.is_empty()
    {
        details.push(format!("models={}", summarize_string_list(models)));
    }

    if let Some(providers) = agent.providers.as_deref()
        && !providers.is_empty()
    {
        details.push(format!("providers={}", summarize_string_list(providers)));
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
        let all = definition::discover(&cwd).await;

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
                Some(AgentSource::Workspace) => Cell::new("workspace").fg(Color::Magenta),
                Some(AgentSource::User) => Cell::new("user").fg(Color::Green),
                Some(AgentSource::Builtin) => Cell::new("builtin").fg(Color::Blue),
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
        let agent = definition::get_by_name(&cwd, &self.name).await?;

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
    async fn skill_discovery_root_for_target(target: &ResolvedValidationTarget) -> Result<PathBuf> {
        match target {
            ResolvedValidationTarget::Path { path, .. } => {
                let base = path
                    .parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .unwrap_or(path);
                match stencila_dirs::closest_workspace_dir(base, false).await {
                    Ok(workspace_dir) => Ok(workspace_dir),
                    Err(_) => Ok(base.to_path_buf()),
                }
            }
            ResolvedValidationTarget::Agent { agent_path, .. } => {
                if agent_path.as_os_str().is_empty() {
                    std::env::current_dir().map_err(Into::into)
                } else {
                    let base = agent_path
                        .parent()
                        .filter(|p| !p.as_os_str().is_empty())
                        .unwrap_or(agent_path);
                    match stencila_dirs::closest_workspace_dir(base, false).await {
                        Ok(workspace_dir) => Ok(workspace_dir),
                        Err(_) => Ok(base.to_path_buf()),
                    }
                }
            }
        }
    }

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
        let agent = definition::get_by_name(&cwd, &self.target).await?;

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

        let agent_path = agent.path().to_path_buf();

        Ok(ResolvedValidationTarget::Agent {
            agent: agent.inner,
            agent_path,
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
        let target = self.resolve_target().await?;
        #[cfg(feature = "skills")]
        let skill_discovery_root = Self::skill_discovery_root_for_target(&target).await?;

        let (agent, dir_name) = match target {
            ResolvedValidationTarget::Path { path, dir_name } => {
                let agent = Self::parse_agent(&path).await?;
                (agent, dir_name)
            }
            ResolvedValidationTarget::Agent {
                agent, dir_name, ..
            } => (agent, dir_name),
        };

        let errors = validate::validate_agent(&agent, dir_name.as_deref());

        #[cfg(feature = "skills")]
        let warnings = { validate::validate_agent_skills(&agent, &skill_discovery_root).await };
        #[cfg(not(feature = "skills"))]
        let warnings: Vec<validate::ValidationWarning> = Vec::new();

        if !warnings.is_empty() {
            message!(
                "⚠️  Agent `{}` has {} warning{}:",
                agent.name,
                warnings.len(),
                plural(warnings.len())
            );
            for warning in &warnings {
                message!("  - {}", warning);
            }
        }

        if errors.is_empty() {
            if warnings.is_empty() {
                message!("🎉 Agent `{}` is valid", agent.name);
            } else {
                message!(
                    "🎉 Agent `{}` is valid with {} warning{}",
                    agent.name,
                    warnings.len(),
                    plural(warnings.len())
                );
            }
            Ok(())
        } else {
            message!(
                "❌ Agent `{}` has {} error{}:",
                agent.name,
                errors.len(),
                plural(errors.len())
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

        let resolved_name = if self.name == crate::DEFAULT_AGENT_NAME {
            crate::convenience::resolve_default_agent_name()
        } else {
            self.name.clone()
        };
        let cwd = std::env::current_dir()?;
        let agent = definition::get_by_name(&cwd, &resolved_name).await?;

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
            tracing::warn!("No API client configured; showing CLI fallback route");
        }

        let effective_providers = agent
            .inner
            .providers
            .clone()
            .or_else(|| resolve_configured_model_providers(&cwd));

        let decision = routing::route_session_explained(
            agent.inner.models.as_deref(),
            effective_providers.as_deref(),
            agent.inner.model_size.as_deref(),
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
            println!("Selection:        {:?}", decision.selection_mechanism);
            if !decision.skipped.is_empty() {
                println!("Skipped models:");
                for skipped in &decision.skipped {
                    println!("  - {}: {:?}", skipped.model, skipped.reason);
                }
            }

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
            let agent = definition::get_by_name(&cwd, &self.name).await?;
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

        // Create session from agent definition and submit.
        // Build a store so checkpoints are written to the workspace DB.
        let cwd = std::env::current_dir()?;
        let store = open_workspace_db_for_agents(&cwd)
            .ok()
            .map(|(s, _db)| Arc::new(s));

        let (_agent, mut session, mut event_rx) = create_session_with_options(
            &self.name,
            CreateSessionOptions {
                store,
                persistence: Some(SessionPersistence::BestEffort),
                ..Default::default()
            },
        )
        .await?;

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
// Sessions subcommand group
// ---------------------------------------------------------------------------

/// Manage agent sessions
#[derive(Debug, Args)]
pub struct Sessions {
    #[command(subcommand)]
    command: SessionsCommand,
}

#[derive(Debug, Subcommand)]
enum SessionsCommand {
    /// List agent sessions
    List(SessionsList),
    /// Show details of a specific session
    Show(SessionsShow),
}

#[derive(Debug, Args)]
struct SessionsList {
    /// Only show resumable sessions
    #[arg(long)]
    resumable: bool,
}

#[derive(Debug, Args)]
struct SessionsShow {
    /// The session ID to show
    session_id: String,
}

impl Sessions {
    async fn run(self) -> Result<()> {
        match self.command {
            SessionsCommand::List(list) => list.run().await,
            SessionsCommand::Show(show) => show.run().await,
        }
    }
}

impl SessionsList {
    async fn run(self) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let (store, _db) = open_workspace_db_for_agents(&cwd)?;
        let filter = filter_sessions_for_list(self.resumable);
        let sessions = store
            .list_sessions(&filter)
            .map_err(|e| eyre::eyre!("Failed to list sessions: {e}"))?;

        if sessions.is_empty() {
            message!("No sessions found.");
            return Ok(());
        }

        // Resolve workflow names for sessions that belong to a workflow run
        let run_ids: Vec<&str> = sessions
            .iter()
            .filter_map(|s| s.workflow_run_id.as_deref())
            .collect();
        let workflow_names = store.resolve_workflow_names(&run_ids);

        let mut table = Tabulated::new();
        table.set_header([
            "ID",
            "Agent",
            "Workflow",
            "State",
            "Resumable",
            "Turns",
            "Updated",
        ]);

        for session in &sessions {
            let row = format_session_list_row(session, &workflow_names);
            table.add_row([
                Cell::new(&row.id).add_attribute(Attribute::Bold),
                Cell::new(&row.agent_name),
                Cell::new(&row.workflow),
                Cell::new(&row.state),
                Cell::new(&row.resumability),
                Cell::new(&row.total_turns),
                Cell::new(&row.updated_at),
            ]);
        }

        table.to_stdout();
        Ok(())
    }
}

impl SessionsShow {
    #[allow(clippy::print_stdout)]
    async fn run(self) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let (store, _db) = open_workspace_db_for_agents(&cwd)?;

        let record = store
            .get_session_by_prefix(&self.session_id)
            .map_err(|e| eyre::eyre!("{e}"))?
            .ok_or_else(|| eyre::eyre!("Session '{}' not found", self.session_id))?;

        let turns = store
            .get_turns(&record.session_id)
            .map_err(|e| eyre::eyre!("Failed to get turns: {e}"))?;

        let detail = format_session_detail(&record, &turns);
        println!("{detail}");
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Session formatting and filtering helpers
//
// Public because they are exercised by integration tests in `tests/`.
// ---------------------------------------------------------------------------

pub struct SessionListRow {
    pub id: String,
    pub agent_name: String,
    pub workflow: String,
    pub state: String,
    pub resumability: String,
    pub total_turns: String,
    pub updated_at: String,
}

pub fn format_session_list_row(
    record: &SessionRecord,
    workflow_names: &std::collections::HashMap<String, String>,
) -> SessionListRow {
    let truncate_len = 8;
    let id = record
        .session_id
        .get(..truncate_len)
        .unwrap_or(&record.session_id)
        .to_string();

    let workflow = record
        .workflow_run_id
        .as_ref()
        .and_then(|run_id| workflow_names.get(run_id).cloned())
        .unwrap_or_default();

    SessionListRow {
        id,
        agent_name: record.agent_name.clone(),
        workflow,
        state: record.state.to_string(),
        resumability: record.resumability.to_string(),
        total_turns: record.total_turns.to_string(),
        updated_at: record.updated_at.clone(),
    }
}

pub fn format_session_detail(record: &SessionRecord, turns: &[Turn]) -> String {
    use std::fmt::Write;

    let mut out = String::new();

    let _ = writeln!(out, "Session:      {}", record.session_id);
    let _ = writeln!(out, "Agent:        {}", record.agent_name);
    let _ = writeln!(out, "Provider:     {}", record.provider_name);
    let _ = writeln!(out, "Model:        {}", record.model_name);
    let _ = writeln!(out, "Backend:      {}", record.backend_kind);
    let _ = writeln!(out, "State:        {}", record.state);
    let _ = writeln!(out, "Resumability: {}", record.resumability);
    let _ = writeln!(out, "Total Turns:  {}", record.total_turns);
    let _ = writeln!(out, "Created:      {}", record.created_at);
    let _ = write!(out, "Updated:      {}", record.updated_at);

    if !turns.is_empty() {
        let _ = write!(out, "\n\nConversation:");
        for turn in turns {
            match turn {
                Turn::User { content, .. } => {
                    let _ = write!(out, "\n  [user] {content}");
                }
                Turn::Assistant { content, .. } => {
                    let _ = write!(out, "\n  [assistant] {content}");
                }
                Turn::ToolResults { results, .. } => {
                    let _ = write!(out, "\n  [tool-results] ({} results)", results.len());
                }
                Turn::System { content, .. } => {
                    let _ = write!(out, "\n  [system] {content}");
                }
                Turn::Steering { content, .. } => {
                    let _ = write!(out, "\n  [steering] {content}");
                }
            }
        }
    }

    out
}

pub fn filter_sessions_for_list(resumable: bool) -> ListSessionsFilter {
    ListSessionsFilter {
        resumable: if resumable { Some(true) } else { None },
        ..Default::default()
    }
}

pub fn open_workspace_db_for_agents(
    workspace_root: &Path,
) -> Result<(AgentSessionStore, stencila_db::WorkspaceDb)> {
    let db_path = workspace_root.join(".stencila").join("db.sqlite3");
    let db = stencila_db::WorkspaceDb::open(&db_path)
        .map_err(|e| eyre::eyre!("Failed to open workspace DB: {e}"))?;
    db.migrate("agents", AGENT_MIGRATIONS)
        .map_err(|e| eyre::eyre!("Failed to apply agent migrations: {e}"))?;
    let store = AgentSessionStore::new(db.connection().clone());
    Ok((store, db))
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

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
