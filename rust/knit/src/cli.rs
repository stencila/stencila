use std::path::PathBuf;

use common::{
    clap::{self, Parser},
    eyre::Result,
};
use format::Format;

use super::knit;

/// Knit a document
///
/// Additional, passthrough arguments can be associated with each output e.g.
///
/// ```sh
/// stencila knit report.tex report.docx --reference-doc=reference.docx out.pdf
/// ```
///
/// Note that passthrough arguments, must come after the output path they are
/// associated with, must begin with a hyphen and not contain any spaces (e.g.
/// use `--reference-doc=reference.docx`, not `--reference-doc reference.docx`).
#[derive(Default, Debug, Parser)]
pub struct Knit {
    /// The path of the input document
    input: PathBuf,

    /// The paths of the output documents and any associated passthrough options
    #[arg(allow_hyphen_values = true)]
    outputs: Vec<String>,

    /// The format of the input document
    ///
    /// If not supplied is inferred from the filename extension of the input.
    #[arg(long, short)]
    from: Option<Format>,

    /// The format of the output documents
    ///
    /// If not supplied is inferred from the filename extension of each output.
    #[arg(long, short)]
    to: Option<Format>,
}

impl Knit {
    pub async fn run(self) -> Result<()> {
        knit::knit(self.input, self.outputs, self.from, self.to).await
    }
}
