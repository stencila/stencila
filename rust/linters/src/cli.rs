use std::path::PathBuf;

use cli_utils::{
    AsFormat, Code, ToStdout,
    color_print::cstr,
    tabulated::{Attribute, Cell, Color, Tabulated},
};
use stencila_linter::{
    Format, LinterAvailability, LinterSpecification, LintingOptions, NodeType,
    common::{
        clap::{self, Args, Parser, Subcommand},
        eyre::{Result, bail},
        itertools::Itertools,
        serde_yaml,
        tokio::fs::read_to_string,
    },
};

use crate::{lint, list};

/// Manage linters
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all available linters</dim>
  <b>stencila linters</b>

  <dim># Lint a file using a linter</dim>
  <b>stencila linters lint</b> <g>script.py</g>
"
);

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
    Lint(Lint),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            return List::default().run().await;
        };

        match command {
            Command::List(list) => list.run().await,
            Command::Lint(lint) => lint.run().await,
        }
    }
}

/// List the linters available
#[derive(Debug, Default, Args)]
#[command(after_long_help = LIST_AFTER_LONG_HELP)]
struct List {
    /// Only list linter that support a specific language/format
    #[arg(long, short, alias = "lang", alias = "format")]
    language: Option<Format>,

    /// Only list linter that support a specific node type
    #[arg(long, short)]
    node_type: Option<NodeType>,

    /// Output the list as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

impl List {
    async fn run(self) -> Result<()> {
        let list = list().await.into_iter().filter(|linter| {
            if let Some(format) = &self.language
                && !linter.formats().contains(format)
            {
                return false;
            }

            if let Some(node_type) = &self.node_type
                && !linter.node_types().contains(node_type)
            {
                return false;
            }

            true
        });

        if let Some(format) = self.r#as {
            let list = list
                .map(|linter| LinterSpecification::from(linter.as_ref()))
                .collect_vec();

            Code::new_from(format.into(), &list)?.to_stdout();

            return Ok(());
        }

        let mut table = Tabulated::new();
        table.set_header(["Name", "Languages/formats", "Node types", "Availability"]);

        for linter in list {
            let name = linter.name();
            let formats = linter
                .formats()
                .iter()
                .map(|format| format.name())
                .join(", ");
            let node_types = linter
                .node_types()
                .iter()
                .map(|node_type| node_type.to_string())
                .join(", ");
            let availability = linter.availability();

            table.add_row([
                Cell::new(name).add_attribute(Attribute::Bold),
                Cell::new(formats),
                Cell::new(node_types),
                Cell::new(availability).fg(match availability {
                    LinterAvailability::Available => Color::Green,
                    LinterAvailability::Installable => Color::Cyan,
                    LinterAvailability::Unavailable => Color::Grey,
                }),
            ]);
        }

        table.to_stdout();

        Ok(())
    }
}

pub static LIST_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all available linters</dim>
  <b>stencila linters list</b>

  <dim># List only Python linters</dim>
  <b>stencila linters list</b> <c>--lang</c> <g>py</g>

  <dim># List only citation linters</dim>
  <b>stencila linters list</b> <c>--node-type</c> <g>Citation</g>

  <dim># Output linter list as YAML</dim>
  <b>stencila linters list</b> <c>--as</c> <g>yaml</g>
"
);

/// Lint a file
///
/// Mainly intended for testing linters during development of Stencila. To lint
/// a document use `stencila lint`.
#[derive(Debug, Args)]
#[command(after_long_help = LINT_AFTER_LONG_HELP)]
struct Lint {
    /// The file to lint
    file: PathBuf,

    /// The name of the linter to use
    #[arg(long, short)]
    linter: Option<String>,

    /// Format the content of the file
    #[arg(long)]
    format: bool,

    /// Fix warnings and errors in the file where possible
    #[arg(long)]
    fix: bool,
}

impl Lint {
    #[allow(clippy::print_stderr)]
    async fn run(self) -> Result<()> {
        if !self.file.exists() {
            bail!("File does not exist: `{}`", self.file.display())
        }
        let file = self.file.canonicalize()?;

        let format = Format::from_path(&file);
        let content = read_to_string(&file).await?;

        let outputs = lint(
            &content,
            Some(&file),
            LintingOptions {
                linter: self.linter,
                format: Some(format.clone()),
                should_fix: self.fix,
                should_format: self.format,
                ..Default::default()
            },
        )
        .await?;

        for output in outputs {
            if let Some(authors) = output.authors {
                eprintln!("Linter:\n");
                Code::new(Format::Yaml, &serde_yaml::to_string(&authors)?).to_stdout();
            }

            if let Some(code) = output.code {
                eprintln!("Formatted and/or fixed code:\n");
                Code::new(format.clone(), &code).to_stdout();
            }

            if let Some(messages) = output.messages {
                eprintln!("Diagnostic messages:\n");
                Code::new(Format::Yaml, &serde_yaml::to_string(&messages)?).to_stdout();
            }
        }

        Ok(())
    }
}

pub static LINT_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Lint a Python file</dim>
  <b>stencila linters lint</b> <g>script.py</g>

  <dim># Lint and format a JavaScript file</dim>
  <b>stencila linters lint</b> <g>app.js</g> <c>--format</c>

  <dim># Lint and fix issues where possible</dim>
  <b>stencila linters lint</b> <g>code.r</g> <c>--fix</c>

  <dim># Lint with both formatting and fixing</dim>
  <b>stencila linters lint</b> <g>code.py</g> <c>--format</c> <c>--fix</c>
"
);
