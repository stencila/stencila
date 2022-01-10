#![recursion_limit = "256"]

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    #[cfg(feature = "cli")]
    stencila::cli::main().await?;

    #[cfg(all(not(feature = "cli"), feature = "server"))]
    stencila::server::main().await?;

    Ok(())
}
