use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use serde::Serialize;

use crate::{errors::InternalError, server::ServerState};

/// Messages sent from server to client for site live reload
#[derive(Clone, Serialize)]
#[serde(tag = "type")]
pub enum SiteMessage {
    /// Config file changed
    ConfigChange,
    /// Theme file changed
    ThemeChange,
    /// Site files changed
    SiteChange {
        /// Paths that changed (relative to site root)
        paths: Vec<String>,
    },
    /// An error occurred
    Error {
        /// The error message
        message: String,
    },
}

/// Handle WebSocket upgrade request for site watching
#[tracing::instrument(skip_all)]
pub async fn websocket_handler(
    State(state): State<ServerState>,
    ws: WebSocketUpgrade,
) -> Result<Response, InternalError> {
    Ok(ws.on_upgrade(move |socket| handle_site_socket(socket, state)))
}

/// Handle the WebSocket connection for site live reload
#[tracing::instrument(skip_all)]
async fn handle_site_socket(mut socket: WebSocket, state: ServerState) {
    tracing::trace!("Site WebSocket connection established");

    // If site_notify is set, use that instead of watching files directly.
    // This is used by site preview to notify after re-rendering completes.
    if let Some(notify_sender) = &state.site_notify {
        let mut notify_receiver = notify_sender.subscribe();

        loop {
            match notify_receiver.recv().await {
                Ok(msg) => {
                    if let Ok(json) = serde_json::to_string(&msg)
                        && socket.send(Message::Text(json.into())).await.is_err()
                    {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    break;
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    // Skip lagged messages and continue
                    continue;
                }
            }
        }

        tracing::trace!("Site WebSocket connection closed");
        return;
    }

    let config = match stencila_config::get() {
        Ok(c) => c,
        Err(e) => {
            let msg = SiteMessage::Error {
                message: format!("Failed to load config: {e}"),
            };
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send(Message::Text(json.into())).await;
            }
            return;
        }
    };

    let site_root = config
        .site
        .as_ref()
        .and_then(|s| s.root.as_ref())
        .map(|r| state.dir.join(r))
        .unwrap_or_else(|| state.dir.clone());

    // Watch config file for changes
    let mut config_receiver = stencila_config::watch(&state.dir).await.ok().flatten();

    // Watch workspace theme (theme.css) for changes
    let mut theme_receiver = match stencila_themes::watch(None, Some(&state.dir)).await {
        Ok(rx) => rx,
        Err(error) => {
            tracing::warn!("Failed to watch theme: {error}");
            None
        }
    };

    // Watch site root for file changes
    let mut site_receiver = match stencila_site::watch(&site_root, None).await {
        Ok(rx) => Some(rx),
        Err(error) => {
            tracing::warn!("Failed to watch site: {error}");
            None
        }
    };

    // Forward changes to WebSocket
    loop {
        tokio::select! {
            // Handle config changes (if watching)
            result = async {
                match config_receiver.as_mut() {
                    Some(rx) => rx.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                match result {
                    Some(Ok(_config)) => {
                        let msg = SiteMessage::ConfigChange;
                        if let Ok(json) = serde_json::to_string(&msg)
                            && socket.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                    }
                    Some(Err(e)) => {
                        tracing::warn!("Config watch error: {e}");
                    }
                    None => {
                        config_receiver = None; // Channel closed
                    }
                }
            }

            // Handle theme changes (if watching)
            result = async {
                match theme_receiver.as_mut() {
                    Some(rx) => rx.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                match result {
                    Some(Ok(_theme)) => {
                        let msg = SiteMessage::ThemeChange;
                        if let Ok(json) = serde_json::to_string(&msg)
                            && socket.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                    }
                    Some(Err(e)) => {
                        tracing::warn!("Theme watch error: {e}");
                    }
                    None => {
                        theme_receiver = None; // Channel closed
                    }
                }
            }

            // Handle site file changes (if watching)
            result = async {
                match site_receiver.as_mut() {
                    Some(rx) => rx.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                match result {
                    Some(event) => {
                        let msg = SiteMessage::SiteChange {
                            paths: event.paths.iter()
                                .filter_map(|p| p.to_str())
                                .map(String::from)
                                .collect(),
                        };
                        if let Ok(json) = serde_json::to_string(&msg)
                            && socket.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                    }
                    None => {
                        site_receiver = None; // Channel closed
                    }
                }
            }

            // Exit if all watchers are gone
            _ = std::future::pending::<()>(), if config_receiver.is_none() && site_receiver.is_none() && theme_receiver.is_none() => {
                break;
            }
        }
    }

    tracing::trace!("Site WebSocket connection closed");
}
