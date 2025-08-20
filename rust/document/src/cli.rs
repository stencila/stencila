use std::{
    env::current_dir,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use ask::{Answer, ask_with_default};
use cli_utils::{
    AsFormat, Code, ToStdout,
    color_print::cstr,
    tabulated::{Attribute, Cell, Color, Tabulated},
};
use codecs::{EncodeOptions, LossesResponse};
use common::{
    chrono::TimeDelta,
    chrono_humanize,
    clap::{self, Parser},
    eyre::{Report, Result, bail},
    futures::future::try_join_all,
    itertools::Itertools,
    reqwest::Url,
    tokio::fs::create_dir_all,
    tracing,
};
use dirs::{CreateStencilaDirOptions, STENCILA_DIR, closest_workspace_dir, stencila_dir_create};
use format::Format;
use kernels::Kernels;
use node_diagnostics::{Diagnostic, DiagnosticKind, DiagnosticLevel};
use schema::{
    Article, Block, Collection, CreativeWorkVariant, ExecutionBounds, Node, NodeId, NodeType,
};

use crate::track::DocumentRemote;

use super::{Document, track::DocumentTrackingStatus};

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

/// Rebuild a workspace database
#[derive(Debug, Parser)]
#[command(after_long_help = REBUILD_AFTER_LONG_HELP)]
pub struct Rebuild {
    /// The workspace directory to rebuild the database for
    ///
    /// Defaults to the current directory.
    #[arg(default_value = ".")]
    dir: PathBuf,
}

pub static REBUILD_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Rebuild database for current workspace</dim>
  <b>stencila rebuild</>

  <dim># Rebuild database for specific workspace</dim>
  <b>stencila rebuild</> <g>./my-project</>

<bold><b>Note</b></bold>
  This recreates the workspace database from scratch,
  re-scanning all tracked documents and their metadata.
  Use this if the database becomes corrupted or outdated.
"
);

impl Rebuild {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        Document::tracking_rebuild(&self.dir).await
    }
}

/// Query a workspace database
#[derive(Debug, Parser)]
#[command(after_long_help = QUERY_AFTER_LONG_HELP)]
pub struct Query {
    /// The document, or document database, to query
    ///
    /// Use the path to a file to create a temporary database for that
    /// file to query.
    input: String,

    /// The DocsQL or Cypher query to run
    ///
    /// If the query begins with the word `MATCH` it will be assumed to be cypher.
    /// Use the `--cypher` flag to force this.
    query: Option<String>,

    /// The path of the file to output the result to
    ///
    /// If not supplied the output content is written to `stdout`.
    output: Option<String>,

    /// The directory from which the closest workspace should be found
    ///
    /// Only applies when `input` is `.` or `workspace`
    /// Defaults to the current directory. Use this option if wanting
    /// to query a database outside of the current workspace, or if
    /// not in a workspace.
    #[arg(long, default_value = ".")]
    dir: PathBuf,

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
  <dim># Query the workspace database</dim>
  <b>stencila query</> <y>\"workspace.paragraphs()\"</>

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
        if !self.dir.exists() {
            bail!("Directory `{}` does not exist", self.dir.display())
        }

        // Shift positional arguments to handle case when two or less provided
        let (document, query, output) = match (self.query, self.output) {
            // Three args
            (Some(query), Some(output)) => (Some(self.input), query, Some(output)),
            // Two args
            (Some(query), None) => {
                if PathBuf::from(&self.input).exists() {
                    // Input exists of the filesystem so use args as provided
                    (Some(self.input), query, None)
                } else {
                    // "Shift" args to right so that second one is the output
                    (None, self.input, Some(query))
                }
            }
            // One arg
            (None, ..) => (None, self.input, None),
        };

        let db = match document {
            Some(..) => "document",
            None => "workspace", // TODO: add --db option for specifying db when using cypher
        };

        let (language, code) = if self.cypher || query.to_lowercase().starts_with("match ") {
            ("docsdb", format!("// @{db}\n{query}"))
        } else {
            ("docsql", query.to_string())
        };

        let (nodes, messages, ..) = if let Some(path) = document.map(PathBuf::from) {
            // Open the document and execute within its kernels
            let document = Document::open(&path, None).await?;
            if !self.no_compile {
                document.compile().await?;
            }
            let mut kernels = document.kernels.write().await;
            kernels.execute(&code, Some(language)).await?
        } else {
            // Create an "orphan" set of kernels (not bound to a document)
            let mut kernels = Kernels::new(ExecutionBounds::Main, &self.dir, None);
            kernels.execute(&code, Some(language)).await?
        };

        // Display any messages as a diagnostic
        for msg in messages {
            Diagnostic {
                node_type: NodeType::CodeChunk,
                node_id: NodeId::null(),
                level: DiagnosticLevel::from(&msg.level),
                kind: DiagnosticKind::Execution,
                error_type: msg.error_type.clone(),
                message: msg.message.clone(),
                format: None,
                code: Some(query.to_string()),
                code_location: msg.code_location.clone(),
            }
            .to_stderr_pretty("<code>", &query, &None)
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

        if let Some(output) = output.map(PathBuf::from) {
            // If output is defined then encode to file
            codecs::to_path(
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
            let content = codecs::to_string(
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

    /// The URL of the remote to track
    url: Option<Url>,
}

pub static TRACK_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Start tracking a local document</dim>
  <b>stencila track</> <g>document.md</>

  <dim># Track a document with remote URL</dim>
  <b>stencila track</> <g>document.md</> <g>https://example.com/api/docs/123</>

  <dim># Track multiple documents</dim>
  <b>stencila track</> <g>*.md</>

<bold><b>Note</b></bold>
  Tracking enables version control, synchronization,
  and change detection for documents. Remote URLs allow
  syncing with external systems.
"
);

impl Track {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        if let Some(url) = self.url {
            let already_tracked = Document::track_path_with_remote(
                &self.file,
                (url.clone(), DocumentRemote::default()),
            )
            .await?;
            eprintln!(
                "🟢 {} tracking {url} for `{}`",
                if already_tracked {
                    "Continued"
                } else {
                    "Started"
                },
                self.file.display()
            );
        } else {
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
        }

        Ok(())
    }
}

/// Stop tracking a document
#[derive(Debug, Parser)]
#[command(after_long_help = UNTRACK_AFTER_LONG_HELP)]
pub struct Untrack {
    /// The path of the file to stop tracking
    ///
    /// Use "deleted" to untrack all files that have been deleted.
    file: PathBuf,

    /// The URL of the remote to stop tracking
    url: Option<Url>,
}

pub static UNTRACK_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Stop tracking a document</dim>
  <b>stencila untrack</> <g>document.md</>

  <dim># Stop tracking a remote URL for a document</dim>
  <b>stencila untrack</> <g>document.md</> <g>https://example.com/api/docs/123</>

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
        } else if let Some(url) = self.url {
            Document::untrack_remote(&self.file, &url).await?;
            eprintln!("🟥 Stopped tracking {url} for `{}`", self.file.display());
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

/// Get the tracking status of documents
#[derive(Debug, Parser)]
#[command(after_long_help = STATUS_AFTER_LONG_HELP)]
pub struct Status {
    /// The paths of the files to get status for
    files: Vec<PathBuf>,

    /// Output the status as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

pub static STATUS_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Show status of all tracked documents</dim>
  <b>stencila status</>

  <dim># Show status of specific documents</dim>
  <b>stencila status</> <g>document.md</> <g>report.md</>

  <dim># Output status as JSON</dim>
  <b>stencila status</> <c>--as</> <g>json</>

<bold><b>Status Information</b></bold>
  Shows modification times, storage status, and sync
  information for tracked documents and their remotes.
"
);

impl Status {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        let statuses = if self.files.is_empty() {
            // No paths provided, so get statuses from tracking dir
            match Document::tracking_all(&current_dir()?).await? {
                Some(statuses) => statuses,
                None => {
                    eprintln!("✖️ Current folder is not being tracked by Stencila");
                    return Ok(());
                }
            }
        } else {
            // Check that each path exists
            for path in self.files.iter() {
                if !path.exists() {
                    bail!("Path does not exist: {}", path.display())
                }
            }

            // Get status of each file
            let futures = self.files.into_iter().map(|path| async {
                let status = Document::tracking_path(&path).await?;
                Ok::<_, Report>((path, status))
            });
            let statuses = try_join_all(futures).await?;
            statuses
                .into_iter()
                .flat_map(|(path, tracking)| {
                    tracking.and_then(|tracking| tracking.1.map(|entry| (path, entry)))
                })
                .collect()
        };

        if let Some(format) = self.r#as {
            // Return early with formatted list
            Code::new_from(format.into(), &statuses)?.to_stdout();
            return Ok(());
        }

        let workspace_dir = closest_workspace_dir(&current_dir()?, false).await?;

        let mut table = Tabulated::new();
        table.set_header([
            "File\n↳ Remote",
            "Status",
            "Modified\n",
            "Stored\n↳ Pulled",
            "Added\n↳ Pushed",
        ]);

        for (path, entry) in statuses {
            let (status, modified_at) = entry.status(&workspace_dir, &path);

            use DocumentTrackingStatus::*;
            let (attr, color) = match status {
                Unsupported => (Attribute::Dim, Color::DarkGrey),
                Deleted => (Attribute::Bold, Color::Red),
                Synced => (Attribute::Bold, Color::Green),
                Ahead => (Attribute::Bold, Color::Yellow),
                Behind => (Attribute::Bold, Color::Red),
            };

            table.add_row([
                Cell::new(path.to_string_lossy()).add_attribute(attr),
                // Currently, only show status for deleted files
                Cell::new(if matches!(status, DocumentTrackingStatus::Deleted) {
                    status.to_string()
                } else {
                    String::new()
                })
                .fg(color),
                // Do not show modified time if deleted
                Cell::new(if matches!(status, DocumentTrackingStatus::Deleted) {
                    String::new()
                } else {
                    humanize_timestamp(modified_at)?
                }),
                Cell::new(humanize_timestamp(entry.stored_at)?),
                Cell::new(humanize_timestamp(entry.added_at)?),
            ]);

            for (url, remote) in entry.remotes.iter().flatten() {
                table.add_row([
                    Cell::new(format!("↳ {url}")),
                    Cell::new(""),
                    Cell::new(""),
                    Cell::new(format!("↳ {}", humanize_timestamp(remote.pulled_at)?))
                        .add_attribute(if remote.pulled_at.is_none() {
                            Attribute::Dim
                        } else {
                            Attribute::Reset
                        }),
                    Cell::new(format!("↳ {}", humanize_timestamp(remote.pushed_at)?))
                        .add_attribute(if remote.pushed_at.is_none() {
                            Attribute::Dim
                        } else {
                            Attribute::Reset
                        }),
                ]);
            }
        }

        table.to_stdout();

        Ok(())
    }
}

fn humanize_timestamp(time: Option<u64>) -> Result<String> {
    use chrono_humanize::{Accuracy, HumanTime, Tense};

    let Some(time) = time else {
        return Ok(String::from("-"));
    };

    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs()
        .saturating_sub(time);
    let time_delta = TimeDelta::seconds(seconds as i64);

    let mut string = HumanTime::from(time_delta).to_text_en(Accuracy::Rough, Tense::Present);
    if string == "now" {
        string.insert_str(0, "just ");
    } else {
        string.push_str(" ago");
    }

    Ok(string)
}
