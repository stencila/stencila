use std::net::SocketAddr;

use axum::{Router, Server};
use eyre::Result;

// Re-exports for consumers of this crate
pub use axum;
pub use portpicker;
pub use serde_json;

/// Serve with graceful shutdown on Ctrl+C
pub async fn serve_gracefully(ip: [u8; 4], port: u16, router: Router) -> Result<()> {
    let addr = SocketAddr::from((ip, port));
    let server = Server::bind(&addr).serve(router.into_make_service());
    let graceful = server.with_graceful_shutdown(shutdown_signal());
    graceful.await?;
    
    Ok(())
}

/// Wait until the Ctrl+C signal is sent
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C signal handler");
}
