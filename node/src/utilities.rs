//! Internal utility functions

use napi::{Error, Status};

use common::eyre;

pub(crate) fn generic_failure(report: eyre::Report) -> Error {
    Error::new(Status::GenericFailure, report.to_string())
}

#[allow(unused)]
pub(crate) fn invalid_arg(report: eyre::Report) -> Error {
    Error::new(Status::InvalidArg, report.to_string())
}
