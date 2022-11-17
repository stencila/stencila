use common::strum::EnumString;
use graph::{PlanOrdering, PlanScope};
use node_patch::Patch;
use uuids::uuid_family;

/// When requests should be fulfilled
#[derive(Debug, Clone, Copy, EnumString)]
#[strum(crate = "common::strum")]
pub enum When {
    Now,
    Soon,
    Never,
}

impl When {
    pub fn no_later_than(&mut self, other: When) {
        match (&self, other) {
            (When::Soon, When::Now) | (When::Never, When::Soon) | (When::Never, When::Now) => {
                *self = other
            }
            _ => {}
        }
    }
}

uuid_family!(RequestId, "re");

/// An internal request to patch a document
#[derive(Debug)]
pub struct PatchRequest {
    pub ids: Vec<RequestId>,
    pub patch: Patch,
    pub when: When,
    pub compile: When,
    pub execute: When,
    pub write: When,
}

impl PatchRequest {
    pub fn new(
        ids: Vec<RequestId>,
        patch: Patch,
        when: When,
        compile: When,
        execute: When,
        write: When,
    ) -> Self {
        Self {
            ids,
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
    pub ids: Vec<RequestId>,
    pub when: When,
    pub execute: When,
    pub write: When,
    pub start: Option<String>,
}

impl CompileRequest {
    pub fn new(
        ids: Vec<RequestId>,
        when: When,
        execute: When,
        write: When,
        start: Option<String>,
    ) -> Self {
        Self {
            ids,
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
    pub ids: Vec<RequestId>,
    pub when: When,
    pub write: When,
    pub start: Option<String>,
    pub ordering: Option<PlanOrdering>,
    pub max_concurrency: Option<usize>,
}

impl ExecuteRequest {
    pub fn new(
        ids: Vec<RequestId>,
        when: When,
        write: When,
        start: Option<String>,
        ordering: Option<PlanOrdering>,
        max_concurrency: Option<usize>,
    ) -> Self {
        Self {
            ids,
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
    pub ids: Vec<RequestId>,
    pub start: Option<String>,
    pub scope: Option<PlanScope>,
}

impl CancelRequest {
    pub fn new(ids: Vec<RequestId>, start: Option<String>, scope: Option<PlanScope>) -> Self {
        Self { ids, start, scope }
    }
}

/// An internal request to write the document (e.g. after patching)
#[derive(Debug)]
pub struct WriteRequest {
    pub ids: Vec<RequestId>,
    pub when: When,
}

impl WriteRequest {
    pub fn new(ids: Vec<RequestId>, when: When) -> Self {
        Self { ids, when }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub request_id: RequestId,
}

impl Response {
    pub fn new(request_id: RequestId) -> Self {
        Self { request_id }
    }
}
