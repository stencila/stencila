use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// A server error which, as well as having the usual message string,
/// has a HTTP status code
pub struct ServerError {
    pub status: StatusCode,
    pub message: String,
}

impl ServerError {
    pub fn new<S: AsRef<str>>(status: StatusCode, message: S) -> Self {
        Self {
            status,
            message: message.as_ref().to_string(),
        }
    }
}

/// Convert an `eyre::Report` to a `ServerError`
///
/// This makes it possible to use `?` to automatically convert an `eyre::Report`
/// returned by an inner function call to a `ServerError`
impl From<eyre::Report> for ServerError {
    fn from(report: eyre::Report) -> Self {
        ServerError::new(StatusCode::INTERNAL_SERVER_ERROR, report.to_string())
    }
}

/// Convert a `ServerError` to a `Response`
///
/// This allows us to return a `Result<_,ServerError>` from handler functions and
/// have them be returned to the client as a JSON response consistent with the error
/// responses returned by Stencila Cloud.
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "error": {
                "message": self.message
            }
        }));
        (self.status, body).into_response()
    }
}
