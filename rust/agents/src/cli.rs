//! CLI for managing agent definitions.
//!
//! Provides `stencila agents` subcommands: list, show, validate, create.

use std::{path::PathBuf, process::exit};

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

use crate::{agent_def::{self, AgentSource}, agent_validate};

/// Manage agent definitions
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
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
  <b>stencila agents create</> <g>my-agent</>

  <dim># Create a new agent in user config</dim>
  <b>stencila agents create</> <g>my-agent</> <c>--user</>
"
);

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
    Show(Show),
    Validate(Validate),
    Create(Create),
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
        }

        Ok(())
    }
}

/// List available agents
///
/// Shows agents from workspace `.stencila/agents/` and user config
/// `~/.config/stencila/agents/`. Use `--source` to filter by source.
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
  <b>stencila agents create</> <g>my-agent</>

  <dim># Create a new agent in user config</dim>
  <b>stencila agents create</> <g>my-agent</> <c>--user</>
"
);

impl Create {
    #[allow(clippy::print_stderr)]
    async fn run(self) -> Result<()> {
        // Validate the name first
        let name_errors = agent_validate::validate_name(&self.name);
        if !name_errors.is_empty() {
            message!("⚠️  Invalid agent name `{}`:", self.name);
            for error in &name_errors {
                message!("  - {}", error);
            }
            exit(1)
        }

        let agents_dir = if self.user {
            stencila_dirs::get_app_dir(stencila_dirs::DirType::Agents, true)?
        } else {
            let cwd = std::env::current_dir()?;
            agent_def::closest_agents_dir(&cwd, true).await?
        };

        let agent_dir = agents_dir.join(&self.name);

        if agent_dir.exists() {
            eyre::bail!(
                "Agent `{}` already exists at `{}`",
                self.name,
                agent_dir.display()
            );
        }

        tokio::fs::create_dir_all(&agent_dir).await?;

        let agent_md = agent_dir.join("AGENT.md");
        let template = format!(
            "\
---
name: {name}
description: {description}
---

TODO: Add instructions for this agent.
",
            name = self.name,
            description = self.description
        );
        tokio::fs::write(&agent_md, template).await?;

        message!(
            "✨ Created agent `{}` at `{}`",
            self.name,
            agent_dir.display()
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
    /// Resolve the target to an AGENT.md path and optional directory name
    async fn resolve_target(&self) -> Result<(PathBuf, Option<String>)> {
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
            return Ok((path, dir_name));
        }

        // If target is a directory (containing AGENT.md)
        if path.is_dir() {
            let agent_md = path.join("AGENT.md");
            if agent_md.exists() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).map(String::from);
                return Ok((agent_md, dir_name));
            }
            eyre::bail!("No AGENT.md found in directory `{}`", path.display());
        }

        // Otherwise, treat as an agent name — look up across all sources
        let cwd = std::env::current_dir()?;
        let agent = agent_def::get_by_name(&cwd, &self.target).await?;
        let agent_path = agent.path().to_path_buf();
        let dir_name = agent_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(String::from);
        Ok((agent_path, dir_name))
    }

    async fn run(self) -> Result<()> {
        let (agent_md, dir_name) = self.resolve_target().await?;

        let content = tokio::fs::read_to_string(&agent_md).await?;
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
            eyre::bail!("Failed to parse `{}` as an Agent", agent_md.display());
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
