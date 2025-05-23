use std::path::PathBuf;

use codecs::LossesResponse;
use common::{
    clap::{self, Parser},
    eyre::Result,
};
use format::Format;

use crate::options::{DecodeOptions, EncodeOptions, StripOptions};

/// Merge changes from an edited document into the original
#[derive(Debug, Parser)]
pub struct Cli {
    /// The edited version of the document
    edited: PathBuf,

    /// The original version of the document
    original: PathBuf,

    /// The unedited version of the document
    ///
    /// This should be in the same format as the edited version.
    unedited: Option<PathBuf>,

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
            workdir,
            ..
        } = self;

        let decode_options = self.decode_options.build(
            None,
            StripOptions::default(),
            LossesResponse::Debug,
            None,
            Vec::new(),
        );

        let encode_options = self.encode_options.build(
            Some(&edited),
            Some(&original),
            None,
            Format::Markdown,
            StripOptions::default(),
            LossesResponse::Debug,
            None,
            Vec::new(),
        );

        codecs::merge(
            &edited,
            &original,
            unedited.as_deref(),
            decode_options,
            encode_options,
            workdir,
        )
        .await
    }
}
