use common::{
    clap::{self, Parser, ValueEnum},
    eyre::Result,
    itertools::Itertools,
    strum::{EnumIter, IntoEnumIterator},
    tokio,
};

use schema_gen::schemas::Schemas;

/// Generate things from the Stencila Schema
#[derive(Parser)]
struct Args {
    /// What to generate
    ///
    /// A space separated list of outputs.
    /// Defaults to everything.
    what: Vec<What>,
}

// The possible things to generate
#[derive(Clone, Copy, ValueEnum, EnumIter)]
#[strum(crate = "common::strum")]
enum What {
    Docs,
    JsonLd,
    JsonSchema,
    Rust,
    Typescript,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let whats = if args.what.is_empty() {
        What::iter().collect_vec()
    } else {
        args.what
    };

    let mut schemas = Schemas::read().await?;
    schemas.check()?;
    schemas.extend()?;
    schemas.expand()?;

    use What::*;
    for what in whats {
        match what {
            Docs => schemas.docs().await?,
            JsonLd => schemas.json_ld().await?,
            JsonSchema => schemas.json_schema().await?,
            Rust => schemas.rust().await?,
            Typescript => schemas.typescript().await?,
        }
    }

    Ok(())
}
