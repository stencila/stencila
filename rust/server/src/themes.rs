use axum::{
    extract::{Query, State, WebSocketUpgrade, ws::WebSocket},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::{errors::InternalError, server::ServerState};

/// Query parameters for theme WebSocket connection
#[derive(Debug, Deserialize)]
pub struct ThemeParams {
    /// The type of theme: "workspace" or "user"
    #[serde(rename = "theme-type")]
    theme_type: String,

    /// The name of the theme (required for user themes, ignored for workspace)
    #[serde(rename = "theme-name")]
    theme_name: Option<String>,
}

/// Messages sent from server to client
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ThemeMessage {
    /// Theme CSS has been updated
    ThemeUpdate {
        /// The type of theme
        theme_type: String,
        /// The normalized CSS content
        content: String,
        /// The theme name (if available)
        name: Option<String>,
    },
    /// An error occurred
    Error {
        /// The error message
        message: String,
    },
}

/// Handle WebSocket upgrade request for theme watching
#[tracing::instrument(skip_all)]
pub async fn websocket_handler(
    State(ServerState { dir, .. }): State<ServerState>,
    ws: WebSocketUpgrade,
    Query(params): Query<ThemeParams>,
) -> Result<Response, InternalError> {
    // Validate theme type
    if params.theme_type != "workspace" && params.theme_type != "user" {
        return Ok((StatusCode::BAD_REQUEST, "Invalid theme-type").into_response());
    }

    // User themes require a name
    if params.theme_type == "user" && params.theme_name.is_none() {
        return Ok((StatusCode::BAD_REQUEST, "User themes require theme-name").into_response());
    }

    Ok(ws.on_upgrade(move |socket| handle_theme_socket(socket, params, dir)))
}

/// Handle the WebSocket connection for theme watching
#[tracing::instrument(skip(socket))]
async fn handle_theme_socket(mut socket: WebSocket, params: ThemeParams, dir: std::path::PathBuf) {
    tracing::trace!("Theme WebSocket connection established");

    // Determine theme name based on type
    let theme_name = if params.theme_type == "workspace" {
        None
    } else {
        params.theme_name.as_deref()
    };

    // Start watching the theme
    let mut theme_receiver = match stencila_themes::watch(theme_name, Some(&dir)).await {
        Ok(Some(receiver)) => receiver,
        Ok(None) => return,
        Err(error) => {
            tracing::error!("Failed to watch theme: {error}");
            let msg = ThemeMessage::Error {
                message: format!("Failed to watch theme: {error}"),
            };
            let _ = socket
                .send(serde_json::to_string(&msg).unwrap_or_default().into())
                .await;
            return;
        }
    };

    tracing::debug!("Watching theme: {theme_name:?}");

    // Listen for theme updates and forward to WebSocket
    while let Some(theme_result) = theme_receiver.recv().await {
        match theme_result {
            Ok(updated) => {
                tracing::debug!("Theme updated, sending to client");

                let message = ThemeMessage::ThemeUpdate {
                    theme_type: params.theme_type.clone(),
                    content: updated.content,
                    name: updated.name,
                };

                if let Ok(json) = serde_json::to_string(&message) {
                    if socket.send(json.into()).await.is_err() {
                        tracing::debug!("Client disconnected");
                        break;
                    }
                } else {
                    tracing::error!("Failed to serialize theme message");
                }
            }
            Err(error) => {
                tracing::error!("Theme watch error: {error}");
                let msg = ThemeMessage::Error {
                    message: format!("Theme error: {error}"),
                };
                if let Ok(json) = serde_json::to_string(&msg)
                    && socket.send(json.into()).await.is_err()
                {
                    break;
                }
            }
        }
    }

    tracing::trace!("Theme WebSocket connection closed");
}
