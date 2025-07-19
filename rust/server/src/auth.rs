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
    access_token: Option<String>,
    otc: Option<String>,
}

/// Login to the server with an access token
///
/// Currently this simply provides a way of removing the need for the `access_token` query
/// parameter by setting a cookie and redirecting to the desired path. In the future, it
/// my include a login form which will be presented when no access token is supplied.
#[tracing::instrument(skip_all)]
pub async fn callback(
    State(state): State<ServerState>,
    Query(query): Query<AuthQuery>,
) -> Response {
    let Some(server_access_token) = state.access_token else {
        return (
            StatusCode::UNAUTHORIZED,
            "Route is only permitted with secured server",
        )
            .into_response();
    };

    let Some(access_token) = query.access_token else {
        return (StatusCode::UNAUTHORIZED, "Access token required").into_response();
    };

    if access_token != server_access_token {
        return (StatusCode::UNAUTHORIZED, "Invalid access token").into_response();
    }

    let Some(otc) = query.otc else {
        return (StatusCode::BAD_REQUEST, "One-time code is required").into_response();
    };

    let Ok(response) = Client::new()
        .post(&format!("{}/access-tokens/otc", cloud::base_url()))
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
