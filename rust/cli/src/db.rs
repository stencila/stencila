use std::{env::current_dir, path::PathBuf};

use clap::{Parser, Subcommand};
use eyre::Result;
use itertools::Itertools;

use stencila_cli_utils::{Code, ToStdout, color_print::cstr, message};
use stencila_codecs::{EncodeOptions, LossesResponse};
use stencila_dirs::closest_stencila_dir;
use stencila_document::Document;
use stencila_format::Format;
use stencila_kernels::Kernels;
use stencila_node_db::cli::{Migrate, Migrations, New};
use stencila_node_diagnostics::{Diagnostic, DiagnosticKind, DiagnosticLevel};
use stencila_schema::{
    Article, Block, Collection, CreativeWorkVariant, ExecutionBounds, Node, NodeId, NodeType,
};

use crate::options::{DecodeOptions, StripOptions};

/// Manage the workspace and other document databases
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Run pending migrations on workspace database</dim>
  <b>stencila db migrate</b>

  <dim># Check migration status</dim>
  <b>stencila db migrations status</b>

  <dim># Validate migrations without applying</dim>
  <b>stencila db migrate --dry-run</b>

  <dim># Work with a specific database</dim>
  <b>stencila db migrate /path/to/database.db</b>
"
);

#[derive(Debug, Subcommand)]
enum Command {
    New(New),
    Add(Add),
    Remove(Remove),
    Query(Query),
    Migrate(Migrate),
    Migrations(Migrations),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Command::New(new) => new.run().await,
            Command::Add(add) => add.run().await,
            Command::Remove(remove) => remove.run().await,
            Command::Query(query) => query.run().await,
            Command::Migrate(migrate) => migrate.run().await,
            Command::Migrations(migrations) => migrations.run().await,
        }
    }
}

/// Add documents to the workspace database
#[derive(Debug, Parser)]
#[command(after_long_help = ADD_AFTER_LONG_HELP)]
pub struct Add {
    /// The documents to add to the workspace database
    #[arg(num_args = 1.., required = true)]
    documents: Vec<String>,

    #[command(flatten)]
    decode_options: DecodeOptions,

    /// The tool to use for decoding inputs
    ///
    /// Only supported for formats that use alternative external tools for decoding and ignored otherwise.
    #[arg(long, alias = "to-tool")]
    tool: Option<String>,

    /// Arguments to pass through to the tool using for decoding
    ///
    /// Only supported for formats that use external tools for decoding and ignored otherwise.
    #[arg(last = true, allow_hyphen_values = true)]
    tool_args: Vec<String>,

    /// Do not canonicalize the document
    #[arg(long)]
    no_canonicalize: bool,
}

pub static ADD_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Add a single document to workspace database</dim>
  <b>stencila db add</> <g>document.md</>

  <dim># Add multiple local Markdown documents</dim>
  <b>stencila db add</> <g>*.md</> <g>docs/*.md</>

  <dim># Add all local Markdown documents</dim>
  <b>stencila db add</> <g>**/*.md</>

  <dim># Add a bioRxiv preprint using its DOI</dim>
  <b>stencila db add</> <g>https://doi.org/10.1101/2021.11.24.469827</>

  <dim># Add specific pages from a PDF document</dim>
  <b>stencila db add</> <g>report.pdf</> <c>--pages</> <g>1,3,5-10</>

  <dim># Add PDF excluding cover and appendix pages</dim>
  <b>stencila db add</> <g>book.pdf</> <c>--pages</> <g>2-</> <c>--exclude-pages</> <g>50-</>

  <dim># Add only even pages from a document</dim>
  <b>stencila db add</> <g>manuscript.pdf</> <c>--pages</> <g>even</>

<bold><b>Note</b></bold>
  This adds documents to the workspace database for
  indexing and querying. Files must be within the
  workspace directory to be added. Page selection
  options are available for multi-page formats like PDF.
"
);

impl Add {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        let Some(first_doc) = self.documents.first() else {
            tracing::warn!("No documents provided");
            return Ok(());
        };

        let first_path = PathBuf::from(first_doc);

        let base_path = if first_path.exists() {
            first_path.clone()
        } else {
            current_dir()?
        };
        let stencila_dir = closest_stencila_dir(&base_path, true).await?;

        let decode_options: stencila_codecs::DecodeOptions = self
            .decode_options
            .build(Some(&first_path), StripOptions::default())
            .with_tool(self.tool, self.tool_args);

        Document::add_docs(
            &stencila_dir,
            &self.documents,
            Some(decode_options),
            !self.no_canonicalize,
        )
        .await
    }
}

/// Remove documents from the workspace database
#[derive(Debug, Parser)]
#[clap(alias = "rm")]
#[command(after_long_help = REMOVE_AFTER_LONG_HELP)]
pub struct Remove {
    /// The document to remove from the workspace database
    #[arg(num_args = 1.., required = true)]
    documents: Vec<String>,
}

pub static REMOVE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Remove a document from workspace database</dim>
  <b>stencila db remove</> <g>document.md</>

  <dim># Remove multiple documents</dim>
  <b>stencila db remove</> <g>*.md</> <g>docs/*.md</>

  <dim># Use the rm alias</dim>
  <b>stencila db rm</> <g>old-document.md</>

<bold><b>Note</b></bold>
  This removes documents from the workspace database
  but does not delete the actual files. The files
  will no longer be indexed or queryable.
"
);

impl Remove {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        let Some(first_doc) = self.documents.first() else {
            tracing::warn!("No documents provided");
            return Ok(());
        };

        let first_path = PathBuf::from(first_doc);
        let base_path = if first_path.exists() {
            first_path
        } else {
            current_dir()?
        };

        let stencila_dir = closest_stencila_dir(&base_path, false).await?;
        Document::remove_docs(&stencila_dir, &self.documents).await
    }
}

/// Query a workspace database
#[derive(Debug, Parser)]
#[command(after_long_help = QUERY_AFTER_LONG_HELP)]
pub struct Query {
    /// The DocsQL or Cypher query to run
    ///
    /// If the query begins with the word `MATCH` it will be assumed to be cypher.
    /// Use the `--cypher` flag to force this.
    query: String,

    /// The path of the file to output the result to
    ///
    /// If not supplied the output content is written to `stdout`.
    output: Option<String>,

    /// Use Cypher as the query language (instead of DocsQL the default)
    #[arg(long, short)]
    cypher: bool,

    /// The format to output the result as
    ///
    /// Defaults to inferring the format from the file name extension
    /// of the `output`. If no `output` is supplied, defaults to JSON.
    /// See `stencila codecs list` for available formats.
    #[arg(long, short)]
    to: Option<Format>,

    /// Use compact form of encoding if possible
    ///
    /// Use this flag to produce the compact forms of encoding (e.g. no indentation)
    /// which are supported by some formats (e.g. JSON, HTML).
    #[arg(long, conflicts_with = "pretty")]
    compact: bool,

    /// Use a "pretty" form of encoding if possible
    ///
    /// Use this flag to produce pretty forms of encoding (e.g. indentation)
    /// which are supported by some formats (e.g. JSON, HTML).
    #[arg(long, short, conflicts_with = "compact")]
    pretty: bool,
}

pub static QUERY_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Query the workspace database</dim>
  <b>stencila db query</> <y>\"workspace.paragraphs()\"</>

  <dim># Use Cypher query language</dim>
  <b>stencila db query</> <c>--cypher</> <y>\"MATCH (h:Heading) WHERE h.level = 1 RETURN h\"</>
"
);

impl Query {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        let query = self.query.clone();
        let output = self.output.clone();

        let (language, code) = if self.cypher || self.query.to_lowercase().starts_with("match ") {
            ("docsdb", ["// @document\n", &query].concat())
        } else {
            ("docsql", query.clone())
        };

        // Create an "orphan" set of kernels (not bound to a document)
        let mut kernels = Kernels::new(ExecutionBounds::Main, &current_dir()?, None);
        let (nodes, messages, ..) = kernels.execute(&code, Some(language)).await?;

        // Display any messages as a diagnostic
        for msg in messages {
            Diagnostic {
                node_type: NodeType::CodeChunk,
                node_id: NodeId::null(),
                node_property: None,
                level: DiagnosticLevel::from(&msg.level),
                kind: DiagnosticKind::Execution,
                error_type: msg.error_type.clone(),
                message: msg.message.clone(),
                format: None,
                code: Some(query.to_string()),
                code_location: msg.code_location.clone(),
            }
            .to_stderr("<code>", &query, &None)
            .ok();
        }

        if nodes.is_empty() {
            message("🔍 No nodes matching query");
            return Ok(());
        }

        let node = if nodes.len() == 1 {
            nodes[0].clone()
        } else if nodes.iter().all(|node| node.node_type().is_creative_work()) {
            Node::Collection(Collection::new(
                nodes
                    .into_iter()
                    .map(TryInto::<CreativeWorkVariant>::try_into)
                    .try_collect()?,
            ))
        } else if nodes.iter().all(|node| node.node_type().is_block()) {
            Node::Article(Article::new(
                nodes
                    .into_iter()
                    .map(TryInto::<Block>::try_into)
                    .try_collect()?,
            ))
        } else {
            tracing::warn!(
                "Nodes are not all creative works or blocks, so returning only the first"
            );
            nodes[0].clone()
        };

        let compact = self
            .compact
            .then_some(true)
            .or(self.pretty.then_some(false));

        if let Some(output) = output.map(PathBuf::from) {
            // If output is defined then encode to file
            stencila_codecs::to_path(
                &node,
                &output,
                Some(EncodeOptions {
                    format: self.to,
                    compact,
                    losses: LossesResponse::Debug,
                    ..Default::default()
                }),
            )
            .await?;
        } else if let (Node::Datatable(dt), None) = (&node, &self.r#to) {
            // If node is datatable and no output format is defined, pretty print it
            dt.to_stdout()
        } else {
            // Otherwise print using output format, defaulting to Markdown
            let format = self.r#to.unwrap_or(Format::Markdown);
            let content = stencila_codecs::to_string(
                &node,
                Some(EncodeOptions {
                    format: Some(format.clone()),
                    compact,
                    losses: LossesResponse::Debug,
                    ..Default::default()
                }),
            )
            .await?;
            Code::new(format, &content).to_stdout();
        }

        Ok(())
    }
}
