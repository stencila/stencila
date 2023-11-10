use std::{
    error,
    fmt::{self, Display},
};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use common::tracing;

/// An internal server error
#[derive(Debug)]
pub(crate) struct InternalError;

impl InternalError {
    pub fn new<T: Display>(error: T) -> Self {
        tracing::trace!("{error}");
        Self
    }
}

impl Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InternalError")
    }
}

impl error::Error for InternalError {}

impl IntoResponse for InternalError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
    }
}
