use std::path::PathBuf;

use common::{
    clap::{self, Parser},
    eyre::Result,
};
use format::Format;

use crate::options::{DecodeOptions, EncodeOptions, StripOptions};

/// Reverse changes from an edited document into the original
#[derive(Debug, Parser)]
pub struct Cli {
    /// The edited version of the document
    edited: PathBuf,

    /// The original source of the document
    ///
    /// This file may be in a different same format to the edited version.
    #[arg(long)]
    original: Option<PathBuf>,

    /// The unedited version of the document
    ///
    /// This file should be in the same format as the edited version.
    #[arg(long)]
    unedited: Option<PathBuf>,

    /// The commit at which the edited document was generated
    /// from the original
    #[arg(long)]
    commit: Option<String>,

    /// Do not rebase edits using the unedited version of the document
    #[arg(long)]
    no_rebase: bool,

    #[command(flatten)]
    decode_options: DecodeOptions,

    #[command(flatten)]
    encode_options: EncodeOptions,

    /// The working directory for the merge
    ///
    /// This directory is used to create temporary, intermediate versions of the
    /// document used during the merge. By default a temporary directory is used
    /// but this option is useful if you want to inspect those intermediate files.
    #[arg(long, hide = true)]
    workdir: Option<PathBuf>,
}

impl Cli {
    #[allow(clippy::print_stderr)]
    pub async fn run(self) -> Result<()> {
        let decode_options = self
            .decode_options
            .build(Some(&self.edited), StripOptions::default());

        let encode_options = self.encode_options.build(
            Some(&self.edited),
            self.original.as_deref(),
            Format::Markdown,
            StripOptions::default(),
        );

        let modified_files = codecs::merge(
            &self.edited,
            self.original.as_deref(),
            self.unedited.as_deref(),
            self.commit.as_deref(),
            !self.no_rebase,
            decode_options,
            encode_options,
            self.workdir,
        )
        .await?;

        eprintln!(
            "📝 Merge completed successfully, {}",
            match modified_files.len() {
                0 => "no changes detected".to_string(),
                1 => "1 file modified".to_string(),
                count => format!("{count} files modified"),
            }
        );

        Ok(())
    }
}
