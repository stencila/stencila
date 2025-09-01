use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};


use crate::server::ServerState;

#[derive(Deserialize)]
pub struct LoginQuery {
    sst: Option<String>,
    next: Option<String>,
}

/// Login to the server with an access token
///
/// Currently this simply provides a way of removing the need for the `sst` query
/// parameter by setting a cookie and redirecting to the desired path. In the future, it
/// my include a login form which will be presented when no access token is supplied.
#[tracing::instrument(skip_all)]
pub async fn login(
    State(state): State<ServerState>,
    cookies: Cookies,
    Query(query): Query<LoginQuery>,
) -> Response {
    let next = query.next.as_deref().unwrap_or("/");

    if let Some(server_token) = state.server_token {
        // Ensure access token is correct
        if query.sst != Some(server_token.clone()) {
            return (StatusCode::UNAUTHORIZED, "Invalid server token").into_response();
        }

        // Set the access token as a cookie. Setting path is
        // important so that the cookie is sent for all routes
        // including document websocket connections
        let mut cookie = Cookie::new("sst", server_token);
        cookie.set_path("/");
        cookies.add(cookie);
    }

    Redirect::temporary(next).into_response()
}
