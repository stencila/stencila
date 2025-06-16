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
    original: Option<PathBuf>,

    /// The unedited version of the document
    ///
    /// This should be in the same format as the edited version.
    unedited: Option<PathBuf>,

    /// The commit at which the edited document was generated
    /// from the original
    #[arg(long)]
    commit: Option<String>,

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
    pub async fn run(self) -> Result<()> {
        let Self {
            edited,
            original,
            unedited,
            commit,
            workdir,
            ..
        } = self;

        let decode_options =
            self.decode_options
                .build(Some(&edited), StripOptions::default(), None, Vec::new());

        let encode_options = self.encode_options.build(
            Some(&edited),
            original.as_deref(),
            Format::Markdown,
            StripOptions::default(),
            None,
            Vec::new(),
        );

        codecs::reverse(
            &edited,
            original.as_deref(),
            unedited.as_deref(),
            commit.as_deref(),
            decode_options,
            encode_options,
            workdir,
        )
        .await
    }
}
