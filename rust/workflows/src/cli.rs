//! CLI for managing workflow definitions.
//!
//! Provides `stencila workflows` subcommands: list, show, validate, create.

use std::{path::PathBuf, process::exit};

use clap::{Args, Parser, Subcommand};
use eyre::{Result, bail};
use inflector::Inflector;
use tokio::fs::{create_dir_all, read_to_string, write};

use stencila_cli_utils::{
    AsFormat, Code, ToStdout,
    color_print::cstr,
    message,
    tabulated::{Attribute, Cell, Tabulated},
};
use stencila_codecs::{DecodeOptions, EncodeOptions, Format};
use stencila_schema::{Node, NodeType};

use crate::{workflow_def, workflow_validate};

/// Manage workflow definitions
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all workflows</dim>
  <b>stencila workflows</>

  <dim># Show details about a specific workflow</dim>
  <b>stencila workflows show</> <g>data-pipeline</>

  <dim># Validate a workflow by name, directory, or file path</dim>
  <b>stencila workflows validate</> <g>data-pipeline</>

  <dim># Create a new workflow in the workspace</dim>
  <b>stencila workflows create</> <g>my-workflow</> <y>\"A multi-stage data pipeline\"</>

  <dim># Run a workflow</dim>
  <b>stencila workflows run</> <g>code-review</>

  <dim># Run a workflow with a goal override</dim>
  <b>stencila workflows run</> <g>code-review</> <c>--goal</> <y>\"Implement login feature\"</>
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

/// List available workflows
///
/// Shows workflows from `.stencila/workflows/`.
#[derive(Default, Debug, Args)]
#[command(after_long_help = LIST_AFTER_LONG_HELP)]
struct List {
    /// Output the list as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

pub static LIST_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all workflows in table format</dim>
  <b>stencila workflows list</>

  <dim># Output workflows as JSON</dim>
  <b>stencila workflows list</> <c>--as</> <g>json</>
"
);

impl List {
    async fn run(self) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let list = workflow_def::discover(&cwd).await;

        if let Some(format) = self.r#as {
            Code::new_from(format.into(), &list)?.to_stdout();
            return Ok(());
        }

        let mut table = Tabulated::new();
        table.set_header(["Name", "Description", "Goal"]);

        for wf in list {
            let goal = wf.goal.as_deref().unwrap_or("-");

            table.add_row([
                Cell::new(&wf.name).add_attribute(Attribute::Bold),
                Cell::new(&wf.description),
                Cell::new(goal),
            ]);
        }

        table.to_stdout();

        Ok(())
    }
}

/// Show a workflow
///
/// Displays the full content and metadata of a specific workflow.
#[derive(Debug, Args)]
#[command(after_long_help = SHOW_AFTER_LONG_HELP)]
struct Show {
    /// The name of the workflow to show
    name: String,

    /// The format to show the workflow in
    #[arg(long, short, default_value = "md")]
    r#as: Format,
}

pub static SHOW_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Show a workflow as Markdown</dim>
  <b>stencila workflows show</> <g>data-pipeline</>

  <dim># Show a workflow as JSON</dim>
  <b>stencila workflows show</> <g>data-pipeline</> <c>--as</> <g>json</>
"
);

impl Show {
    async fn run(self) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let wf = workflow_def::get_by_name(&cwd, &self.name).await?;

        let content = stencila_codecs::to_string(
            &Node::Workflow(wf.inner),
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

/// Validate a workflow
///
/// Checks that a workflow conforms to naming and property constraint rules,
/// and validates the pipeline DOT if present.
/// Accepts a workflow name, a directory path, or a path to a WORKFLOW.md file.
#[derive(Debug, Args)]
#[command(after_long_help = VALIDATE_AFTER_LONG_HELP)]
struct Validate {
    /// Workflow name, directory path, or WORKFLOW.md path
    target: String,
}

pub static VALIDATE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Validate a workflow by name</dim>
  <b>stencila workflows validate</> <g>data-pipeline</>

  <dim># Validate a workflow directory</dim>
  <b>stencila workflows validate</> <g>.stencila/workflows/data-pipeline</>

  <dim># Validate a WORKFLOW.md file directly</dim>
  <b>stencila workflows validate</> <g>.stencila/workflows/data-pipeline/WORKFLOW.md</>
"
);

impl Validate {
    /// Resolve the target to a WORKFLOW.md path and optional directory name
    async fn resolve_target(&self) -> Result<(PathBuf, Option<String>)> {
        let path = PathBuf::from(&self.target);

        // If target is a path to a WORKFLOW.md file
        if path.is_file()
            && path
                .file_name()
                .is_some_and(|n| n.eq_ignore_ascii_case("WORKFLOW.md"))
        {
            let dir_name = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(String::from);
            return Ok((path, dir_name));
        }

        // If target is a directory (containing WORKFLOW.md)
        if path.is_dir() {
            let workflow_md = path.join("WORKFLOW.md");
            if workflow_md.exists() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).map(String::from);
                return Ok((workflow_md, dir_name));
            }
            bail!("No WORKFLOW.md found in directory `{}`", path.display());
        }

        // Otherwise, treat as a workflow name — look up
        let cwd = std::env::current_dir()?;
        let wf = workflow_def::get_by_name(&cwd, &self.target).await?;
        let wf_path = wf.path().to_path_buf();
        let dir_name = wf_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(String::from);
        Ok((wf_path, dir_name))
    }

    async fn run(self) -> Result<()> {
        let (workflow_md, dir_name) = self.resolve_target().await?;

        let content = read_to_string(&workflow_md).await?;
        let node = stencila_codecs::from_str(
            &content,
            Some(DecodeOptions {
                format: Some(Format::Markdown),
                node_type: Some(NodeType::Workflow),
                ..Default::default()
            }),
        )
        .await?;

        let Node::Workflow(workflow) = node else {
            bail!("Failed to parse `{}` as a Workflow", workflow_md.display());
        };

        let errors = workflow_validate::validate_workflow(&workflow, dir_name.as_deref());

        if errors.is_empty() {
            message!("🎉 Workflow `{}` is valid", workflow.name);
            Ok(())
        } else {
            message!(
                "⚠️  Workflow `{}` has {} error{}:",
                workflow.name,
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

/// Create a new workflow
///
/// Creates a new workflow directory with a template WORKFLOW.md in the
/// workspace's `.stencila/workflows/` directory.
#[derive(Debug, Args)]
#[command(after_long_help = CREATE_AFTER_LONG_HELP)]
struct Create {
    /// The name for the new workflow
    ///
    /// Must be lowercase kebab-case: 1-64 characters, only lowercase alphanumeric
    /// and hyphens, no leading/trailing/consecutive hyphens.
    name: String,

    /// A brief description of the new workflow
    description: String,
}

pub static CREATE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Create a new workflow in the workspace</dim>
  <b>stencila workflows create</> <g>my-workflow</> <y>\"A multi-stage data pipeline\"</>
"
);

impl Create {
    async fn run(self) -> Result<()> {
        let name_errors = workflow_validate::validate_name(&self.name);
        if !name_errors.is_empty() {
            for error in &name_errors {
                message!("  - {}", error);
            }
            bail!("Invalid workflow name `{}`", self.name);
        }

        let cwd = std::env::current_dir()?;
        let workflows_dir = workflow_def::closest_workflows_dir(&cwd, true).await?;
        let wf_dir = workflows_dir.join(&self.name);

        if wf_dir.exists() {
            bail!(
                "Workflow `{}` already exists at `{}`",
                self.name,
                wf_dir.display()
            );
        }

        create_dir_all(&wf_dir).await?;

        let title = self.name.to_title_case();
        let name_underscored = self.name.replace('-', "_");

        let content = format!(
            r#"---
name: {name}
description: {description}
---

# {title}

```dot
digraph {name_underscored} {{
    Start -> Design -> Build -> Test
    Test -> Review       [label="Pass"]
    Test -> Build        [label="Fail"]
    Review -> End        [label="Approve"]
    Review -> Design     [label="Revise"]

    Design [agent="code-planner", prompt="Design the solution for: $goal"]
    Build  [agent="code-engineer", prompt="Implement the design"]
    Test   [agent="code-tester", prompt="Run tests and validate"]
    Review [shape=human]
}}
```
"#,
            name = self.name,
            description = self.description,
            title = title,
            name_underscored = name_underscored,
        );

        let workflow_md = wf_dir.join("WORKFLOW.md");
        write(&workflow_md, content).await?;

        message!(
            "✨ Created workflow `{}` at `{}`",
            self.name,
            wf_dir.display()
        );

        Ok(())
    }
}

/// Run a workflow
///
/// Executes a workflow pipeline. Discovers the workflow by name, parses the
/// DOT pipeline, resolves agents, and runs the pipeline through the attractor
/// engine. Currently uses stub backends that log what they would do.
#[derive(Debug, Args)]
#[command(after_long_help = RUN_AFTER_LONG_HELP)]
struct Run {
    /// The name of the workflow to run
    name: String,

    /// Override the pipeline goal
    ///
    /// If set, replaces the workflow's `goal` field for this run.
    /// The `$goal` variable in node prompts will expand to this value.
    #[arg(long, short)]
    goal: Option<String>,

    /// Directory for run logs and artifacts
    ///
    /// Defaults to a temporary directory.
    #[arg(long)]
    logs_dir: Option<PathBuf>,

    /// Show workflow config and pipeline without executing
    #[arg(long)]
    dry_run: bool,
}

pub static RUN_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Run a workflow</dim>
  <b>stencila workflows run</> <g>code-review</>

  <dim># Run with a goal override</dim>
  <b>stencila workflows run</> <g>code-review</> <c>--goal</> <y>\"Implement login feature\"</>

  <dim># Run with a custom logs directory</dim>
  <b>stencila workflows run</> <g>code-review</> <c>--logs-dir</> <g>./run-logs</>

  <dim># Dry run to see pipeline config</dim>
  <b>stencila workflows run</> <g>code-review</> <c>--dry-run</>
"
);

impl Run {
    #[allow(clippy::print_stdout)]
    async fn run(self) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let mut wf = workflow_def::get_by_name(&cwd, &self.name).await?;

        // Apply goal override if provided
        if let Some(ref goal) = self.goal {
            wf.inner.goal = Some(goal.clone());
        }

        if self.dry_run {
            message!("Workflow: {}", wf.name);
            message!("Description: {}", wf.description);
            if let Some(ref goal) = wf.goal {
                message!("Goal: {}", goal);
            }

            let agents = wf.agent_references();
            if agents.is_empty() {
                message!("Agents: (none)");
            } else {
                message!("Agents: {}", agents.join(", "));
            }

            if let Some(ref pipeline) = wf.pipeline {
                message!("\nPipeline DOT:");
                Code::new(stencila_codecs::Format::Unknown, pipeline).to_stdout();
            } else {
                message!("Pipeline: (none)");
            }

            if let Some(ref stylesheet) = wf.options.model_stylesheet {
                message!("\nModel Stylesheet:");
                Code::new(stencila_codecs::Format::Unknown, stylesheet).to_stdout();
            }

            return Ok(());
        }

        // Determine logs directory
        let logs_dir = if let Some(ref dir) = self.logs_dir {
            tokio::fs::create_dir_all(dir).await?;
            dir.clone()
        } else {
            let tmp = std::env::temp_dir().join(format!("stencila-workflow-{}", self.name));
            tokio::fs::create_dir_all(&tmp).await?;
            tmp
        };

        message!("🚀 Running workflow `{}`", wf.name);
        if let Some(ref goal) = wf.goal {
            message!("   Goal: {}", goal);
        }
        message!("   Logs: {}", logs_dir.display());

        let outcome = crate::workflow_run::run_workflow(&wf, &logs_dir).await?;

        message!(
            "\n{} Workflow `{}` finished (status={})",
            if outcome.status.is_success() {
                "✅"
            } else {
                "❌"
            },
            wf.name,
            outcome.status.as_str()
        );

        if !outcome.notes.is_empty() {
            message!("   Notes: {}", outcome.notes);
        }
        if !outcome.failure_reason.is_empty() {
            message!("   Failure: {}", outcome.failure_reason);
        }

        if !outcome.status.is_success() {
            exit(1);
        }

        Ok(())
    }
}
