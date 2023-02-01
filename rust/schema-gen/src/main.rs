use common::{
    clap::{self, Parser},
    eyre::Result,
    tokio,
};
use schema_gen::schemas::Schemas;

/// Generate things from the Stencila Schema
#[derive(Parser)]
struct Args {
    /// Generate reference docs
    #[arg(short, long, default_value_t = true)]
    docs: bool,

    /// Generate Rust types
    #[arg(short, long, default_value_t = true)]
    rust: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut schemas = Schemas::read().await?;
    schemas.check()?;
    schemas.extend()?;
    schemas.expand()?;

    if args.docs {
        schemas.docs().await?;
    }

    if args.rust {
        schemas.rust().await?;
    }

    Ok(())
}
