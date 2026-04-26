use std::{env::current_dir, path::PathBuf};

use clap::Parser;
use eyre::Result;

use stencila_ask::{Answer, ask_with_default};
use stencila_cli_utils::color_print::cstr;

use super::Document;

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
