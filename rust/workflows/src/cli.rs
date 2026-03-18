//! CLI for managing workflow definitions.
//!
//! Provides `stencila workflows` subcommands: list, show, validate, create, run,
//! runs, resume, save, discard.

use std::{io::IsTerminal, path::PathBuf, process::exit, sync::Arc, time::Instant};

use clap::{Args, Parser, Subcommand};
use eyre::{Result, bail};
use inflector::Inflector;
use tokio::fs::{create_dir_all, read_to_string, write};

use stencila_cli_utils::{
    AsFormat, Code, ToStdout,
    color_print::cstr,
    message,
    tabulated::{Attribute, Cell, Color, Tabulated},
};
use stencila_codecs::{DecodeOptions, EncodeOptions, Format};
use stencila_schema::{Node, NodeType};

use crate::{CliInterviewer, definition, validate};

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

  <dim># List recent workflow runs</dim>
  <b>stencila workflows runs</>

  <dim># Resume the last failed or interrupted run</dim>
  <b>stencila workflows resume</>

  <dim># Resume a specific run by ID</dim>
  <b>stencila workflows resume</> <g>01926f3a-...</>

  <dim># Save an ephemeral workflow</dim>
  <b>stencila workflows save</> <g>my-workflow</>

  <dim># Discard an ephemeral workflow</dim>
  <b>stencila workflows discard</> <g>my-workflow</>
"
);

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
    Show(Show),
    Validate(Validate),
    Create(Create),
    Run(Run),
    Runs(Runs),
    Resume(Resume),
    Save(Save),
    Discard(Discard),
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
            Command::Runs(runs) => runs.run().await?,
            Command::Resume(resume) => resume.run().await?,
            Command::Save(save) => save.run().await?,
            Command::Discard(discard) => discard.run().await?,
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
        let list = definition::discover(&cwd).await;

        if let Some(format) = self.r#as {
            Code::new_from(format.into(), &list)?.to_stdout();
            return Ok(());
        }

        let mut table = Tabulated::new();
        table.set_header(["Name", "Description", "Goal"]);

        for wf in list {
            let goal = wf.goal.as_deref().unwrap_or("-");

            let name_cell = if wf.is_ephemeral() {
                Cell::new(&wf.name).add_attribute(Attribute::Dim)
            } else {
                Cell::new(&wf.name).add_attribute(Attribute::Bold)
            };

            table.add_row([name_cell, Cell::new(&wf.description), Cell::new(goal)]);
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
        let wf = definition::get_by_name(&cwd, &self.name).await?;

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
        let wf = definition::get_by_name(&cwd, &self.target).await?;
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

        let (errors, warnings) = validate::validate_workflow(&workflow, dir_name.as_deref());

        if !warnings.is_empty() {
            message!(
                "⚠️  Workflow `{}` has {} warning{}:",
                workflow.name,
                warnings.len(),
                if warnings.len() > 1 { "s" } else { "" }
            );
            for warning in &warnings {
                message!("  - {}", warning);
            }
        }

        if errors.is_empty() {
            if warnings.is_empty() {
                message!("🎉 Workflow `{}` is valid", workflow.name);
            } else {
                message!(
                    "🎉 Workflow `{}` is valid with {} warning{}",
                    workflow.name,
                    warnings.len(),
                    if warnings.len() > 1 { "s" } else { "" }
                );
            }
            Ok(())
        } else {
            message!(
                "❌ Workflow `{}` has {} error{}:",
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
        let name_errors = validate::validate_name(&self.name);
        if !name_errors.is_empty() {
            for error in &name_errors {
                message!("  - {}", error);
            }
            bail!("Invalid workflow name `{}`", self.name);
        }

        let cwd = std::env::current_dir()?;
        let workflows_dir = definition::closest_workflows_dir(&cwd, true).await?;
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
    Start -> End
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

    /// Show detailed output with prompts and responses
    ///
    /// In verbose mode, each stage shows the agent name, full prompt text,
    /// and full response in a box-drawing tree layout. Without this flag,
    /// a compact progress view with spinners is shown (or plain text when
    /// stderr is not a terminal).
    #[arg(long, short)]
    verbose: bool,

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

  <dim># Dry run to see pipeline config</dim>
  <b>stencila workflows run</> <g>code-review</> <c>--dry-run</>
"
);

impl Run {
    #[allow(clippy::print_stdout, clippy::print_stderr)]
    async fn run(self) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let mut wf = definition::get_by_name(&cwd, &self.name).await?;

        // Apply goal override if provided
        if let Some(ref goal) = self.goal {
            wf.inner.goal = Some(goal.clone());
        }

        // Validate before running so errors and warnings surface early,
        // before time or tokens are spent on execution.
        let dir_name = wf
            .path()
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(String::from);
        let (errors, warnings) = validate::validate_workflow(&wf.inner, dir_name.as_deref());

        for warning in &warnings {
            message!("⚠️  {}", warning);
        }
        if !errors.is_empty() {
            for error in &errors {
                message!("❌ {}", error);
            }
            bail!(
                "Workflow `{}` has {} validation error{}; fix before running",
                wf.name,
                errors.len(),
                if errors.len() > 1 { "s" } else { "" }
            );
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

            if let Some(ref overrides) = wf.options.overrides {
                message!("\nOverrides:");
                Code::new(stencila_codecs::Format::Unknown, overrides).to_stdout();
            }

            return Ok(());
        }

        message!("🚀 Running workflow `{}`", wf.name);
        if let Some(ref goal) = wf.goal {
            message!("   Goal: {}", goal);
        }
        eprintln!();

        let is_tty = std::io::stderr().is_terminal();
        let emitter: Arc<dyn stencila_attractor::events::EventEmitter> = if self.verbose {
            Arc::new(crate::emitters::VerboseEventEmitter::new())
        } else if is_tty {
            Arc::new(crate::emitters::ProgressEventEmitter::new())
        } else {
            Arc::new(crate::emitters::PlainEventEmitter::new())
        };

        let started = Instant::now();
        let interviewer: Arc<dyn stencila_attractor::interviewer::Interviewer> =
            Arc::new(CliInterviewer);
        let options = crate::run::RunOptions {
            emitter,
            interviewer: Some(interviewer),
            run_id_out: None,
        };
        let outcome = crate::run::run_workflow_with_options(&wf, options).await?;
        let elapsed = started.elapsed();

        message!("");
        let time_str = format_elapsed(elapsed);
        if outcome.status.is_success() {
            message!("🎉 Workflow `{}` completed in {}", wf.name, time_str);
        } else {
            message!("❌ Workflow `{}` failed in {}", wf.name, time_str);
        }

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

/// List recent workflow runs
///
/// Shows the most recent workflow runs from the workspace database,
/// including run ID, workflow name, goal, status, and timing.
#[derive(Debug, Args)]
#[command(after_long_help = RUNS_AFTER_LONG_HELP)]
struct Runs {
    /// Maximum number of runs to show
    #[arg(long, short = 'n', default_value = "20")]
    limit: u32,

    /// Only show resumable runs
    #[arg(long)]
    resumable: bool,
}

pub static RUNS_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List the 20 most recent runs</dim>
  <b>stencila workflows runs</>

  <dim># List the 5 most recent runs</dim>
  <b>stencila workflows runs</> <c>-n</> <g>5</>
"
);

impl Runs {
    async fn run(self) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let filter = if self.resumable {
            crate::run::RunListFilter::Resumable
        } else {
            crate::run::RunListFilter::All
        };
        let runs = crate::run::list_runs(&cwd, self.limit, filter).await?;

        if runs.is_empty() {
            if self.resumable {
                message!("No resumable workflow runs found in this workspace");
            } else {
                message!("No workflow runs found in this workspace");
            }
            return Ok(());
        }

        // Compute the shortest unique prefix length for run IDs.
        let ids: Vec<&str> = runs.iter().map(|r| r.run_id.as_str()).collect();
        let prefix_len = shortest_unique_prefix_len(&ids);

        let mut table = Tabulated::new();
        table.set_header(["Run ID", "Workflow", "Goal", "Started", "Status"]);

        for run in &runs {
            let short_id = &run.run_id[..prefix_len.min(run.run_id.len())];

            let status_cell = match run.status.as_str() {
                "success" => Cell::new("success").fg(Color::Green),
                "fail" | "failed" => Cell::new("failed").fg(Color::Red),
                "cancelled" => Cell::new("cancelled").fg(Color::DarkYellow),
                "running" => Cell::new("running").fg(Color::Yellow),
                other => Cell::new(other),
            };

            let goal_preview = if run.goal.chars().count() > 40 {
                let truncated: String = run.goal.chars().take(39).collect();
                format!("{truncated}…")
            } else if run.goal.is_empty() {
                "-".to_string()
            } else {
                run.goal.clone()
            };

            let started = crate::run::humanize_timestamp(&run.started_at);

            table.add_row([
                Cell::new(short_id),
                Cell::new(&run.workflow_name),
                Cell::new(&goal_preview),
                Cell::new(&started),
                status_cell,
            ]);
        }

        table.to_stdout();

        Ok(())
    }
}

/// Compute the shortest prefix length that makes every ID in the list
/// unique. Returns at least 8 (the conventional short-hash length) and
/// at most the length of the longest ID.
fn shortest_unique_prefix_len(ids: &[&str]) -> usize {
    let max_len = ids.iter().map(|id| id.len()).max().unwrap_or(0);
    // Start at 8 chars (conventional short-hash length).
    for len in 8..=max_len {
        let mut seen = std::collections::HashSet::with_capacity(ids.len());
        if ids.iter().all(|id| seen.insert(&id[..len.min(id.len())])) {
            return len;
        }
    }
    max_len
}

/// Resume a failed, cancelled, or interrupted workflow run
///
/// Continues execution of a previously failed, cancelled, or interrupted
/// workflow run from where it left off. The pipeline state (completed
/// nodes, context values, edge traversal history) is restored from the
/// workspace database, and execution resumes at the next unfinished node.
///
/// If no run ID is provided, the most recent resumable run (failed,
/// cancelled, or still marked as running) is used.
#[derive(Debug, Args)]
#[command(after_long_help = RESUME_AFTER_LONG_HELP)]
struct Resume {
    /// The run ID to resume
    ///
    /// If omitted, resumes the most recent failed or interrupted run.
    /// Use `stencila workflows runs` to list recent runs and their IDs.
    run_id: Option<String>,

    /// Show detailed output with prompts and responses
    #[arg(long, short)]
    verbose: bool,

    /// Force resume of a run that is still marked as running
    ///
    /// Use this when a previous run was interrupted without being marked
    /// as failed (e.g. the process was killed). Without this flag,
    /// resuming a "running" run is rejected to avoid conflicts with an
    /// active process.
    #[arg(long)]
    force: bool,
}

pub static RESUME_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Resume the last failed or interrupted run</dim>
  <b>stencila workflows resume</>

  <dim># Resume a specific run by ID</dim>
  <b>stencila workflows resume</> <g>01926f3a-7b2c-7d4e-8f1a-9c3d5e7f0a1b</>

  <dim># Resume with verbose output</dim>
  <b>stencila workflows resume</> <c>--verbose</>

  <dim># List runs to find a run ID, then resume it</dim>
  <b>stencila workflows runs</>
  <b>stencila workflows resume</> <g>01926f3a</>
"
);

impl Resume {
    #[allow(clippy::print_stdout, clippy::print_stderr)]
    async fn run(self) -> Result<()> {
        let cwd = std::env::current_dir()?;

        // Resolve the run ID: use provided ID, or find the last resumable run.
        let run_id = if let Some(ref id) = self.run_id {
            crate::run::resolve_run_id_from_db(&cwd, id).await?
        } else {
            let Some(run) = crate::run::last_resumable_run(&cwd).await? else {
                bail!("No resumable workflow runs found in this workspace");
            };
            run.run_id
        };

        // Look up run info for display.
        let run_info = crate::run::get_run(&cwd, &run_id).await.ok();

        if let Some(ref info) = run_info {
            message!(
                "🔄 Resuming workflow `{}` (run {})",
                info.workflow_name,
                &run_id[..run_id.len().min(8)]
            );
            if !info.goal.is_empty() {
                message!("   Goal: {}", info.goal);
            }
            message!("   Previous status: {}", info.status);
        } else {
            message!("🔄 Resuming run {}", &run_id[..run_id.len().min(8)]);
        }
        message!("");

        let is_tty = std::io::stderr().is_terminal();
        let emitter: Arc<dyn stencila_attractor::events::EventEmitter> = if self.verbose {
            Arc::new(crate::emitters::VerboseEventEmitter::new())
        } else if is_tty {
            Arc::new(crate::emitters::ProgressEventEmitter::new())
        } else {
            Arc::new(crate::emitters::PlainEventEmitter::new())
        };

        let started = Instant::now();
        let interviewer: Arc<dyn stencila_attractor::interviewer::Interviewer> =
            Arc::new(CliInterviewer);
        let options = crate::run::RunOptions {
            emitter,
            interviewer: Some(interviewer),
            run_id_out: None,
        };
        let outcome =
            crate::run::resume_workflow_with_options(&run_id, &cwd, options, self.force).await?;
        let elapsed = started.elapsed();

        eprintln!();
        let time_str = format_elapsed(elapsed);

        let workflow_name = run_info
            .as_ref()
            .map(|i| i.workflow_name.as_str())
            .unwrap_or("workflow");
        if outcome.status.is_success() {
            message!("🎉 Workflow `{}` completed in {}", workflow_name, time_str);
        } else {
            message!("❌ Workflow `{}` failed in {}", workflow_name, time_str);
        }

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

/// Save an ephemeral workflow
///
/// Removes the ephemeral marker from a workflow that was created by an
/// agent, converting it into a permanent workspace workflow.
#[derive(Debug, Args)]
#[command(after_long_help = SAVE_AFTER_LONG_HELP)]
struct Save {
    /// The name of the workflow to save
    name: String,
}

pub static SAVE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Save an ephemeral workflow</dim>
  <b>stencila workflows save</> <g>my-workflow</>
"
);

impl Save {
    async fn run(self) -> Result<()> {
        let cwd = std::env::current_dir()?;
        match definition::save_ephemeral(&cwd, &self.name)? {
            true => {
                message!("Saved workflow `{}`", self.name);
                Ok(())
            }
            false => {
                bail!(
                    "Workflow `{}` is not ephemeral or does not exist",
                    self.name
                );
            }
        }
    }
}

/// Discard an ephemeral workflow
///
/// Removes an ephemeral workflow directory that was created by an agent.
/// Only ephemeral workflows can be discarded; permanent workflows must be
/// deleted manually.
#[derive(Debug, Args)]
#[command(after_long_help = DISCARD_AFTER_LONG_HELP)]
struct Discard {
    /// The name of the workflow to discard
    name: String,
}

pub static DISCARD_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Discard an ephemeral workflow</dim>
  <b>stencila workflows discard</> <g>my-workflow</>
"
);

impl Discard {
    async fn run(self) -> Result<()> {
        let cwd = std::env::current_dir()?;
        match definition::discard_ephemeral(&cwd, &self.name)? {
            true => {
                message!("Discarded workflow `{}`", self.name);
                Ok(())
            }
            false => {
                bail!(
                    "Workflow `{}` is not ephemeral or does not exist",
                    self.name
                );
            }
        }
    }
}

fn format_elapsed(d: std::time::Duration) -> String {
    let secs = d.as_secs_f64();
    if secs < 60.0 {
        format!("{secs:.1}s")
    } else if secs < 3600.0 {
        let mins = (secs / 60.0).floor() as u64;
        let remaining = secs - (mins as f64 * 60.0);
        format!("{mins}m {remaining:.0}s")
    } else {
        let hours = (secs / 3600.0).floor() as u64;
        let remaining_mins = ((secs - (hours as f64 * 3600.0)) / 60.0).floor() as u64;
        format!("{hours}h {remaining_mins}m")
    }
}
