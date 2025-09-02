use std::path::PathBuf;

use clap::Parser;
use eyre::Result;

use cli_utils::color_print::cstr;
use format::Format;

use crate::options::{DecodeOptions, EncodeOptions, StripOptions};

/// Merge changes from another format
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
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

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Merge changes from an edited DOCX back to Stencila Markdown</dim>
  <b>stencila merge</b> <g>edited.docx</g> <c>--original</c> <g>document.smd</g>

  <dim># Merge with both original and unedited versions specified</dim>
  <b>stencila merge</b> <g>edited.docx</g> <c>--original</c> <g>source.qmd</g> <c>--unedited</c> <g>generated.docx</g>

  <dim># Merge changes from a specific Git commit</dim>
  <b>stencila merge</b> <g>edited.docx</g> <c>--original</c> <g>document.myst</g> <c>--commit</c> <g>abc123</g>

  <dim># Merge with custom working directory for inspection</dim>
  <b>stencila merge</b> <g>edited.docx</g> <c>--original</c> <g>document.md</g> <c>--workdir</c> <g>./merge-work</g>
"
);

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

        if let Some(modified_files) = modified_files {
            eprintln!(
                "📝 Merge completed successfully, {}",
                match modified_files.len() {
                    0 => "no changes detected".to_string(),
                    1 => "1 file modified".to_string(),
                    count => format!("{count} files modified"),
                }
            );
        } else {
            eprintln!("🚫 Merge cancelled");
        }

        Ok(())
    }
}
