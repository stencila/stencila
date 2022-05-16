use axum::{routing::get, Router};
use eyre::{bail, Result};
use tokio::{sync::mpsc, task::JoinHandle};
use tower_http::trace::TraceLayer;

use crate::statics::{get_static, STATIC_VERSION};

#[derive(Debug, Default)]
pub struct Server {
    port: u16,

    shutdown_sender: Option<mpsc::Sender<()>>,
}

impl Server {
    /// Create a new server
    pub fn new(port: Option<u16>) -> Result<Self> {
        let port = match port.or_else(portpicker::pick_unused_port) {
            Some(port) => port,
            None => bail!("No unused ports available"),
        };

        Ok(Self {
            port,
            ..Default::default()
        })
    }

    // Get the server's port number
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get the the versioned (i.e prefixed with `/~static/<version>`) for an asset
    pub fn static_path(asset: &str) -> String {
        format!("/~static/{}/{}", STATIC_VERSION, asset)
    }

    /// Start the server
    pub fn start(&mut self) -> Result<JoinHandle<()>> {
        let app = Router::new()
            .route("/~static/*path", get(get_static))
            // TODO: In addition to (instead of?) this port current request logging using custom middleware
            // e.g. https://github.com/tokio-rs/axum/blob/main/examples/error-handling-and-dependency-injection/src/main.rs
            .layer(TraceLayer::new_for_http());

        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], self.port));

        let (shutdown_sender, mut shutdown_receiver) = mpsc::channel::<()>(1);
        self.shutdown_sender = Some(shutdown_sender);

        let task = axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .with_graceful_shutdown(async move {
                shutdown_receiver.recv().await;
            });
        let handle = tokio::spawn(async {
            if let Err(error) = task.await {
                tracing::error!("While running server: {}", error)
            }
        });

        Ok(handle)
    }

    /// Stop the server
    pub fn stop(&mut self) -> Result<()> {
        tracing::debug!("Stopping server");

        if self.shutdown_sender.is_some() {
            // Simply dropping the sender
            self.shutdown_sender = None;
            tracing::trace!("Server stopped successfully");
        } else {
            tracing::info!("Server was already stopped");
        }

        Ok(())
    }
}
