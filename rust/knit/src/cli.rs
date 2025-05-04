use std::path::PathBuf;

use common::{
    clap::{self, Parser},
    eyre::Result,
};
use format::Format;

use super::knit;

/// Knit a document
#[derive(Default, Debug, Parser)]
pub struct Knit {
    /// The path of the input document
    input: PathBuf,

    /// The path of the output document
    output: PathBuf,

    /// The format of the input document
    ///
    /// If not supplied is inferred from the filename extension of the input.
    #[arg(long, short)]
    from: Option<Format>,

    /// The format of the output document
    ///
    /// If not supplied is inferred from the filename extension of the output.
    #[arg(long, short)]
    to: Option<Format>,
}

impl Knit {
    pub async fn run(self) -> Result<()> {
        knit::knit(&self.input, &self.output, self.from, self.to).await
    }
}
