use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};

use common::tracing;

use crate::{errors::InternalError, server::ServerState};

/// Create a router for secrets routes
pub fn router() -> Router<ServerState> {
    Router::new()
        .route("/", get(list_secrets))
        .route("/:name", post(set_secret))
        .route("/:name", delete(delete_secret))
}

/// List secrets
#[tracing::instrument]
async fn list_secrets() -> Result<Response, InternalError> {
    Ok(Json(secrets::list().map_err(InternalError::new)?).into_response())
}

/// Set a secret
#[tracing::instrument]
async fn set_secret(Path(name): Path<String>, value: String) -> Result<Response, InternalError> {
    match secrets::set(&name, &value) {
        Ok(..) => Ok(StatusCode::CREATED.into_response()),
        Err(error) => Ok((StatusCode::BAD_REQUEST, error.to_string()).into_response()),
    }
}

/// Delete a secret
#[tracing::instrument]
async fn delete_secret(Path(name): Path<String>) -> Result<Response, InternalError> {
    match secrets::delete(&name) {
        Ok(..) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(error) => Ok((StatusCode::BAD_REQUEST, error.to_string()).into_response()),
    }
}
