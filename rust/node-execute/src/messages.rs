use common::strum::EnumString;
use graph::{PlanOrdering, PlanScope};
use node_patch::Patch;
use uuids::uuid_family;

uuid_family!(RequestId, "re");

/// When requests should be fulfilled
#[derive(Debug, Clone, Copy, EnumString)]
#[strum(crate = "common::strum")]
pub enum When {
    Now,
    Soon,
    Never,
}

/// An internal request to patch a document
#[derive(Debug)]
pub struct PatchRequest {
    pub id: RequestId,
    pub patch: Patch,
    pub when: When,
    pub compile: When,
    pub execute: When,
    pub write: When,
}

impl PatchRequest {
    pub fn new(patch: Patch, when: When, compile: When, execute: When, write: When) -> Self {
        Self {
            id: RequestId::new(),
            patch,
            when,
            compile,
            execute,
            write,
        }
    }
}

/// An internal request to compile a document
#[derive(Debug)]
pub struct CompileRequest {
    pub id: RequestId,
    pub when: When,
    pub execute: When,
    pub write: When,
    pub start: Option<String>,
}

impl CompileRequest {
    pub fn new(when: When, execute: When, write: When, start: Option<String>) -> Self {
        Self {
            id: RequestId::new(),
            when,
            execute,
            write,
            start,
        }
    }
}

/// An internal request to execute a document
#[derive(Debug)]
pub struct ExecuteRequest {
    pub id: RequestId,
    pub when: When,
    pub write: When,
    pub start: Option<String>,
    pub ordering: Option<PlanOrdering>,
    pub max_concurrency: Option<usize>,
}

impl ExecuteRequest {
    pub fn new(
        when: When,
        write: When,
        start: Option<String>,
        ordering: Option<PlanOrdering>,
        max_concurrency: Option<usize>,
    ) -> Self {
        Self {
            id: RequestId::new(),
            when,
            write,
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

/// An internal request to write the document (e.g. after patching)
#[derive(Debug)]
pub struct WriteRequest {
    pub id: RequestId,
    pub when: When,
}

impl WriteRequest {
    pub fn new(when: When) -> Self {
        Self {
            id: RequestId::new(),
            when,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Response {
    PatchResponse(RequestId),
    CompileResponse(RequestId),
    ExecuteResponse(RequestId),
    CancelResponse(RequestId),
    WriteResponse(RequestId),
}
