#![recursion_limit = "256"]

use common::tokio;

#[tokio::main]
pub async fn main() -> common::eyre::Result<()> {
    #[cfg(feature = "cli")]
    stencila::cli::main().await?;

    #[cfg(all(not(feature = "cli"), feature = "server"))]
    stencila::server::main().await?;

    Ok(())
}
