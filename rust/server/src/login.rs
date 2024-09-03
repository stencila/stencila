use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use tower_cookies::{Cookie, Cookies};

use common::{serde::Deserialize, tracing};

use crate::server::ServerState;

#[derive(Deserialize)]
#[serde(crate = "common::serde")]
pub struct LoginQuery {
    access_token: Option<String>,
    next: Option<String>,
}

/// Login to the server with an access token
///
/// Currently this simply provides a way of removing the need for the `access_token` query
/// parameter by setting a cookie and redirecting to the desired path. In the future, it
/// my include a login form which will be presented when no access token is supplied.
#[tracing::instrument(skip_all)]
pub async fn login(
    State(state): State<ServerState>,
    cookies: Cookies,
    Query(query): Query<LoginQuery>,
) -> Response {
    let next = query.next.as_deref().unwrap_or("/");

    if let Some(token) = state.access_token {
        // Ensure access token is correct
        if query.access_token != Some(token.clone()) {
            return (StatusCode::UNAUTHORIZED, "Invalid access token").into_response();
        }

        // Set the access token as a cookie. Setting path is
        // important so that the cookie is sent for all routes
        // including document websocket connections
        let mut cookie = Cookie::new("access_token", token);
        cookie.set_path("/");
        cookies.add(cookie);
    }

    Redirect::temporary(next).into_response()
}
