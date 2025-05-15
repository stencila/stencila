use std::{
    env::current_dir,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use cli_utils::{
    confirm,
    table::{self, Attribute, Cell, Color},
    AsFormat, Code, ToStdout,
};
use codecs::{EncodeOptions, LossesResponse};
use common::{
    chrono::TimeDelta,
    chrono_humanize,
    clap::{self, Parser},
    eyre::{bail, Report, Result},
    futures::future::try_join_all,
    itertools::Itertools,
    reqwest::Url,
    tokio::fs::create_dir_all,
    tracing,
};
use dirs::{
    closest_stencila_dir, closest_workspace_dir, stencila_dir_create, CreateStencilaDirOptions,
    STENCILA_DIR,
};
use format::Format;
use node_diagnostics::{Diagnostic, DiagnosticKind, DiagnosticLevel};
use schema::{Article, Block, Node, NodeId, NodeType};

use crate::track::DocumentRemote;

use super::{track::DocumentTrackingStatus, Document};

/// Initialize a workspace
#[derive(Debug, Parser)]
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
pub struct Rebuild {
    /// The workspace directory to rebuild the database for
    ///
    /// Defaults to the current directory.
    #[arg(default_value = ".")]
    dir: PathBuf,
}

impl Rebuild {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        Document::tracking_rebuild(&self.dir).await
    }
}

/// Query a workspace database
#[derive(Debug, Parser)]
pub struct Query {
    /// The DocsQL query
    query: String,

    /// The path of the file to output the result to
    ///
    /// If not supplied the output content is written to `stdout`.
    output: Option<PathBuf>,

    /// The database to query
    #[arg(long, default_value = "workspace")]
    db: String,

    /// The directory from which the closest workspace should be found
    ///
    /// Defaults to the current directory. Use this option if wanting
    /// to query a database outside of the current workspace, or if
    /// not in a workspace.
    #[arg(long, default_value = ".")]
    dir: PathBuf,

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

impl Query {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        if !self.dir.exists() {
            bail!("Directory `{}` does not exist", self.dir.display())
        }

        let (kernel, query) = if self.cypher {
            ("docsdb", format!("// @{}\n{}", self.db, self.query))
        } else if !self.query.starts_with(&self.db) {
            ("docsql", format!("{}.{}", self.db, self.query))
        } else {
            ("docsql", self.query)
        };

        // Create a docs kernel and execute query
        let Some(kernel) = kernels::get(kernel).await else {
            bail!("Unable to create `{kernel}` kernel")
        };
        let mut kernel = kernel.create_instance(schema::ExecutionBounds::Box)?;
        kernel.start(&self.dir).await?;
        let (nodes, messages) = kernel.execute(&query).await?;

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
                code: None,
                code_location: msg.code_location.clone(),
            }
            .to_stderr_pretty("<code>", &query, &None)
            .ok();
        }

        let node = if nodes.is_empty() {
            eprintln!("No matching nodes");
            return Ok(());
        } else if nodes.len() == 1 {
            nodes[0].clone()
        } else {
            let blocks = nodes
                .into_iter()
                .map(TryInto::<Block>::try_into)
                .try_collect()?;
            Node::Article(Article::new(blocks))
        };

        if let Some(output) = self.output {
            // If output is defined then encode to file
            codecs::to_path(
                &node,
                &output,
                Some(EncodeOptions {
                    format: self.to,
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
            let compact = self
                .compact
                .then_some(true)
                .or(self.pretty.then_some(false));
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
pub struct Config {
    /// The path to the document to resolve
    file: PathBuf,
}

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
pub struct Track {
    /// The path to the local file to track
    file: PathBuf,

    /// The URL of the remote to track
    url: Option<Url>,
}

impl Track {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        if let Some(url) = self.url {
            let already_tracked =
                Document::track_remote(&self.file, (url.clone(), DocumentRemote::default()))
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
pub struct Untrack {
    /// The path of the file to stop tracking
    file: PathBuf,

    /// The URL of the remote to stop tracking
    url: Option<Url>,
}

impl Untrack {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        if let Some(url) = self.url {
            Document::untrack_remote(&self.file, &url).await?;
            eprintln!("🟥 Stopped tracking {url} for `{}`", self.file.display());
        } else {
            Document::untrack_path(&self.file).await?;
            eprintln!("🟥 Stopped tracking `{}`", self.file.display());
        }

        Ok(())
    }
}

/// Add documents to the workspace database
#[derive(Debug, Parser)]
pub struct Add {
    /// The files to add
    files: Vec<PathBuf>,
}

impl Add {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        let Some(first_path) = self.files.first() else {
            return Ok(());
        };
        let stencila_dir = closest_stencila_dir(first_path, false).await?;

        Document::add_paths(&stencila_dir, &self.files).await
    }
}

/// Remove documents from the workspace database
#[derive(Debug, Parser)]
#[clap(alias = "rm")]
pub struct Remove {
    /// The files to remove
    files: Vec<PathBuf>,
}

impl Remove {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        let Some(first_path) = self.files.first() else {
            return Ok(());
        };
        let stencila_dir = closest_stencila_dir(first_path, false).await?;

        Document::remove_paths(&stencila_dir, &self.files).await
    }
}

/// Move a tracked document
///
/// Moves the document file to the new path (if it still exists at the
/// old path) and updates any tracking information.
#[derive(Debug, Parser)]
#[clap(alias = "mv")]
pub struct Move {
    /// The old path of the file
    from: PathBuf,

    /// The new path of the file
    to: PathBuf,

    /// Overwrite the destination path if it already exists
    #[arg(long, short)]
    force: bool,
}

impl Move {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        if self.to.exists()
            && !self.force
            && !confirm("Destination path already exists, overwrite it?")?
        {
            return Ok(());
        }

        Document::move_path(&self.from, &self.to).await
    }
}

/// Get the tracking status of documents
#[derive(Debug, Parser)]
pub struct Status {
    /// The paths of the files to get status for
    files: Vec<PathBuf>,

    /// Output the status as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

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

        let mut table = table::new();
        table.set_header([
            "File\n↳ Remote",
            "Status",
            "Modified\n",
            "Stored\n↳ Pulled",
            "Upserted\n↳ Pushed",
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
                Cell::new(humanize_timestamp(entry.upserted_at)?),
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
