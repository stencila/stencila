use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};

use common::{reqwest::Client, serde::Deserialize, tracing};
use version::STENCILA_VERSION;

use crate::server::ServerState;

/// Create a router for auth routes
pub fn router() -> Router<ServerState> {
    Router::new().route("/callback", get(callback))
}

#[derive(Deserialize)]
#[serde(crate = "common::serde")]
pub struct AuthQuery {
    sst: Option<String>,
    otc: Option<String>,
}

/// Callback when
#[tracing::instrument(skip_all)]
pub async fn callback(
    State(state): State<ServerState>,
    Query(query): Query<AuthQuery>,
) -> Response {
    let Some(server_token) = state.server_token else {
        return (
            StatusCode::UNAUTHORIZED,
            "Route is only permitted with secured server",
        )
            .into_response();
    };

    let Some(sst) = query.sst else {
        return (StatusCode::UNAUTHORIZED, "Server token required").into_response();
    };

    if sst != server_token {
        return (StatusCode::UNAUTHORIZED, "Invalid server token").into_response();
    }

    let Some(otc) = query.otc else {
        return (StatusCode::BAD_REQUEST, "One-time code is required").into_response();
    };

    let Ok(response) = Client::new()
        .post(format!("{}/access-tokens/otc", cloud::base_url()))
        .header("Content-Type", "application/json")
        .json(&cloud::OtcRequest { otc })
        .send()
        .await
    else {
        return (StatusCode::BAD_REQUEST, "One-time code is required").into_response();
    };

    let response = match cloud::process_response::<cloud::OtcResponse>(response).await {
        Ok(resp) => resp,
        Err(err) => {
            return (
                StatusCode::UNAUTHORIZED,
                format!("Authentication failed: {}", err),
            )
                .into_response();
        }
    };

    if response.token.is_empty() {
        return (
            StatusCode::UNAUTHORIZED,
            "Invalid one-time code response: missing token",
        )
            .into_response();
    }

    if let Err(error) = cloud::signin(&response.token) {
        tracing::error!("Unable to sign in using token: {error}");

        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to sign in using token",
        )
            .into_response();
    }

    if let Some(shutdown_sender) = &state.shutdown_sender {
        tracing::debug!("Sending shutdown signal");
        if let Err(error) = shutdown_sender.send(()).await {
            tracing::error!("Failed to send shutdown signal: {error}");
        } else {
            tracing::debug!("Shutdown signal sent successfully");
        }
    }

    let html = include_str!("auth-success.html").replace("STENCILA_VERSION", STENCILA_VERSION);
    Html::from(html).into_response()
}
