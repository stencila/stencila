use eyre::Result;
use stencila_convert_server::{ServerConfig, app_with_config};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<()> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let config = ServerConfig::from_env();

    let address = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&address).await?;
    tracing::info!("Starting Stencila convert server on {address}");

    axum::serve(listener, app_with_config(config)).await?;

    Ok(())
}
