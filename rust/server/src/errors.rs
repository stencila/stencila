use std::{
    error,
    fmt::{self, Debug, Display},
};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// An internal server error
#[derive(Debug)]
pub(crate) struct InternalError;

impl InternalError {
    /// Create a new internal error
    ///
    /// Creates an error log entry with all the debugging niceties of `eyre`.
    ///
    /// Note that `tower_http::trace::on_failure` will crate a `tracing` log
    /// entry for the 500 code so that is not done here to avoid duplication.
    /// Also `tracing::error("{error:?}")` escape ASCII colors which messes up
    /// the colored display.
    pub fn new<T>(error: T) -> Self
    where
        T: Display + Debug,
    {
        eprintln!("{error:?}\n");
        Self
    }
}

impl Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Do not leak any details in display
        write!(f, "InternalError")
    }
}

impl error::Error for InternalError {}

impl IntoResponse for InternalError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
    }
}
