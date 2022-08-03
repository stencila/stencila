use graph::{PlanOrdering, PlanScope};
use node_patch::Patch;
use uuids::uuid_family;

uuid_family!(RequestId, "re");

/// An internal request to patch a document
#[derive(Debug)]
pub struct PatchRequest {
    pub id: RequestId,
    pub patch: Patch,
    pub compile: bool,
    pub execute: bool,
}

impl PatchRequest {
    pub fn new(patch: Patch, compile: bool, execute: bool) -> Self {
        Self {
            id: RequestId::new(),
            patch,
            compile,
            execute,
        }
    }
}

/// An internal request to compile a document
#[derive(Debug)]
pub struct CompileRequest {
    pub id: RequestId,
    pub execute: bool,
    pub start: Option<String>,
}

impl CompileRequest {
    pub fn new(execute: bool, start: Option<String>) -> Self {
        Self {
            id: RequestId::new(),
            execute,
            start,
        }
    }
}

/// An internal request to execute a document
#[derive(Debug)]
pub struct ExecuteRequest {
    pub id: RequestId,
    pub start: Option<String>,
    pub ordering: Option<PlanOrdering>,
    pub max_concurrency: Option<usize>,
}

impl ExecuteRequest {
    pub fn new(
        start: Option<String>,
        ordering: Option<PlanOrdering>,
        max_concurrency: Option<usize>,
    ) -> Self {
        Self {
            id: RequestId::new(),
            start,
            ordering,
            max_concurrency,
        }
    }
}

/// An internal request to cancel execution of a document
#[derive(Debug)]
pub struct CancelRequest {
    pub id: RequestId,
    pub start: Option<String>,
    pub scope: Option<PlanScope>,
}

impl CancelRequest {
    pub fn new(start: Option<String>, scope: Option<PlanScope>) -> Self {
        Self {
            id: RequestId::new(),
            start,
            scope,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Response {
    PatchResponse(RequestId),
    CompileResponse(RequestId),
    ExecuteResponse(RequestId),
    CancelResponse(RequestId),
}
