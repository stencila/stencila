use std::{env::current_dir, path::PathBuf};

use clap::Parser;
use eyre::{Result, bail};
use itertools::Itertools;
use tokio::fs::create_dir_all;

use stencila_ask::{Answer, ask_with_default};
use stencila_cli_utils::{Code, ToStdout, color_print::cstr};
use stencila_codecs::{EncodeOptions, LossesResponse};
use stencila_dirs::{CreateStencilaDirOptions, STENCILA_DIR, stencila_dir_create};
use stencila_format::Format;
use stencila_node_diagnostics::{Diagnostic, DiagnosticKind, DiagnosticLevel};
use stencila_schema::{Article, Block, Collection, CreativeWorkVariant, Node, NodeId, NodeType};

use super::Document;

/// Initialize a workspace
#[derive(Debug, Parser)]
#[command(after_long_help = INIT_AFTER_LONG_HELP)]
pub struct Init {
    /// The workspace directory to initialize
    ///
    /// Defaults to the current directory.
    #[arg(default_value = ".")]
    dir: PathBuf,

    /// Do not create a `.gitignore` file
    #[arg(long)]
    no_gitignore: bool,
}

pub static INIT_AFTER_LONG_HELP: &str = cstr!(
    "<bold></bold>
  <dim># Initialize current directory as a Stencila workspace</dim>
  <b>stencila init</>

  <dim># Initialize a specific directory</dim>
  <b>stencila init</> <g>./my-project</>

  <dim># Initialize without creating .gitignore</dim>
  <b>stencila init</> <c>--no-gitignore</>

<bold><b>Note</b></bold>
  This creates a .stencila directory for workspace configuration
  and document tracking. A .gitignore file is created by default
  to exclude tracking and cache files.
"
);

impl Init {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        if !self.dir.exists() {
            create_dir_all(&self.dir).await?;
        }

        stencila_dir_create(
            &self.dir.join(STENCILA_DIR),
            CreateStencilaDirOptions {
                gitignore_file: !self.no_gitignore,
                ..Default::default()
            },
        )
        .await?;

        eprintln!(
            "🟢 Initialized document config and tracking for directory `{}`",
            self.dir.display()
        );

        Ok(())
    }
}

/// Query a workspace database
#[derive(Debug, Parser)]
#[command(after_long_help = QUERY_AFTER_LONG_HELP)]
pub struct Query {
    /// The document to query
    ///
    /// Use the path to a file to create a temporary database for that
    /// file to query.
    file: PathBuf,

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

    /// Do not compile the document before querying it
    ///
    /// By default, the document is compiled before it is loaded into
    /// the database. This means that if it has any `IncludeBlock` nodes
    /// that their included content will be included in the database.
    /// Use this flag to turn off this behavior.
    #[arg(long)]
    no_compile: bool,

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
  <dim># Query a specific document</dim>
  <b>stencila query</> <g>article.qmd </><y>\"paragraphs().sample(3)\"</>

  <dim># Query with output to file</dim>
  <b>stencila query</> <g>report.myst</> <y>\"headings(.level == 1)\"</> <g>headings.md</>

  <dim># Use Cypher query language</dim>
  <b>stencila query</> <g>doc.ipynb</> <c>--cypher</> <y>\"MATCH (h:Heading) WHERE h.level = 1 RETURN h\"</>
"
);

impl Query {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        if !self.file.exists() {
            bail!("File `{}` does not exist", self.file.display())
        }

        // Open the document
        let document = Document::open(&self.file, None).await?;
        if !self.no_compile {
            document.compile().await?;
        }

        let (language, code) = if self.cypher || self.query.to_lowercase().starts_with("match ") {
            ("docsdb", ["// @document\n", &self.query].concat())
        } else {
            ("docsql", self.query.clone())
        };

        // Execute within the document's kernels
        let mut kernels = document.kernels.write().await;
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
                code: Some(self.query.to_string()),
                code_location: msg.code_location.clone(),
            }
            .to_stderr("<code>", &self.query, &None)
            .ok();
        }

        if nodes.is_empty() {
            eprintln!("🔍 No nodes matching query");
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

        if let Some(output) = self.output.map(PathBuf::from) {
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

/// Display the configuration for a document
#[derive(Debug, Parser)]
#[command(after_long_help = CONFIG_AFTER_LONG_HELP)]
pub struct Config {
    /// The path to the document to resolve
    file: PathBuf,
}

pub static CONFIG_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Show configuration for a document</dim>
  <b>stencila config</> <g>document.md</>

<bold><b>Note</b></bold>
  Shows both the configuration sources (from workspace,
  user, and document-specific configs) and the final
  merged configuration that will be used for the document.
"
);

impl Config {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        let doc = Document::open(&self.file, None).await?;

        let (config, sources) = doc.config_with_sources().await?;

        Code::new(Format::Markdown, "# Config sources").to_stdout();
        Code::new_from(Format::Yaml, &sources)?.to_stdout();

        Code::new(Format::Markdown, "# Merged config").to_stdout();
        Code::new_from(Format::Yaml, &config)?.to_stdout();

        Ok(())
    }
}

/// Start tracking a document
#[derive(Debug, Parser)]
#[command(after_long_help = TRACK_AFTER_LONG_HELP)]
pub struct Track {
    /// The path to the local file to track
    file: PathBuf,
}

pub static TRACK_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Start tracking a local document</dim>
  <b>stencila track</> <g>document.md</>

  <dim># Track multiple documents</dim>
  <b>stencila track</> <g>*.md</>

<bold><b>Note</b></bold>
  Tracking enables version control and change detection for documents.
  Configure remotes in stencila.toml for synchronization with external systems.
"
);

impl Track {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        let (_, already_tracked, ..) = Document::track_path(&self.file, None, None).await?;
        eprintln!(
            "🟢 {} tracking `{}`",
            if already_tracked {
                "Continued"
            } else {
                "Started"
            },
            self.file.display()
        );

        Ok(())
    }
}

/// Stop tracking a document
#[derive(Debug, Parser)]
#[command(after_long_help = UNTRACK_AFTER_LONG_HELP)]
pub struct Untrack {
    /// The path of the file to stop tracking
    ///
    /// Use "all" to untrack all tracked files.
    file: PathBuf,
}

pub static UNTRACK_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Stop tracking a document</dim>
  <b>stencila untrack</> <g>document.md</>

  <dim># Stop tracking all tracked files</dim>
  <b>stencila untrack <g>all</>

<bold><b>Note</b></bold>
  This removes the document from tracking but does not
  delete the file itself.
"
);

impl Untrack {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        if self.file == PathBuf::from("all") {
            Document::untrack_all(&current_dir()?).await?;
            eprintln!("🟥 Stopped tracking all tracked files");
        } else {
            Document::untrack_path(&self.file).await?;
            eprintln!("🟥 Stopped tracking `{}`", self.file.display());
        }

        Ok(())
    }
}

/// Move a tracked document
///
/// Moves the document file to the new path (if it still exists at the
/// old path) and updates any tracking information.
#[derive(Debug, Parser)]
#[clap(alias = "mv")]
#[command(after_long_help = MOVE_AFTER_LONG_HELP)]
pub struct Move {
    /// The old path of the file
    from: PathBuf,

    /// The new path of the file
    to: PathBuf,

    /// Overwrite the destination path if it already exists
    #[arg(long, short)]
    force: bool,
}

pub static MOVE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Move a tracked document</dim>
  <b>stencila move</> <g>old-name.md</> <g>new-name.md</>

  <dim># Move to a different directory</dim>
  <b>stencila move</> <g>document.md</> <g>docs/document.md</>

  <dim># Force overwrite if destination exists</dim>
  <b>stencila move</> <g>source.md</> <g>target.md</> <c>--force</>

  <dim># Use the mv alias</dim>
  <b>stencila mv</> <g>old.md</> <g>new.md</>

<bold><b>Note</b></bold>
  This updates both the file system and tracking
  information. If the destination already exists,
  you'll be prompted unless --force is used.
"
);

impl Move {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        if self.to.exists()
            && !self.force
            && !ask_with_default(
                "Destination path already exists, overwrite it?",
                Answer::Yes,
            )
            .await?
            .is_yes()
        {
            return Ok(());
        }

        Document::move_path(&self.from, &self.to).await
    }
}

/// Clean the current workspace
///
/// Un-tracks any deleted files and removes any unnecessary cache files, and all
/// artifact directories, from the .stencila folder in the current workspace.
#[derive(Debug, Parser)]
#[command(after_long_help = CLEAN_AFTER_LONG_HELP)]
pub struct Clean;

pub static CLEAN_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Clean the .stencila folder for the current workspace</dim>
  <b>stencila clean</>
"
);

impl Clean {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        Document::clean(&current_dir()?).await
    }
}
