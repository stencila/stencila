use eyre::Result;
use stencila_convert_server::app;
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<()> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Starting Stencila convert server on 0.0.0.0:8080");

    axum::serve(listener, app()).await?;

    Ok(())
}
