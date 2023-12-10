use std::{
    error,
    fmt::{self, Debug, Display},
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
    /// Create a new internal error
    ///
    /// Creates an error log entry with all the debugging niceties
    /// of `eyre`.
    pub fn new<T: Display>(error: T) -> Self
    where
        T: Debug,
    {
        tracing::error!("{error:?}");
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
