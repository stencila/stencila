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

/// A response to an internal request to patch a document
#[derive(Debug)]
pub struct PatchResponse {
    pub id: RequestId,
}

impl PatchResponse {
    pub fn null() -> Self {
        Self {
            id: RequestId("".into()),
        }
    }

    pub fn new(id: RequestId) -> Self {
        Self { id }
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

/// A response to an internal request to compile a document
#[derive(Debug)]
pub struct CompileResponse {
    pub id: RequestId,
}

impl CompileResponse {
    pub fn null() -> Self {
        Self {
            id: RequestId("".into()),
        }
    }

    pub fn new(id: RequestId) -> Self {
        Self { id }
    }
}

/// An internal request to execute a document
#[derive(Debug)]
pub struct ExecuteRequest {
    pub id: RequestId,
    pub start: Option<String>,
}

impl ExecuteRequest {
    pub fn new(start: Option<String>) -> Self {
        Self {
            id: RequestId::new(),
            start,
        }
    }
}

/// A response to an internal request to execute a document
#[derive(Debug)]
pub struct ExecuteResponse {
    pub id: RequestId,
}

impl ExecuteResponse {
    pub fn null() -> Self {
        Self {
            id: RequestId("".into()),
        }
    }

    pub fn new(id: RequestId) -> Self {
        Self { id }
    }
}

/// An internal request to cancel execution of a document
#[derive(Debug)]
pub struct CancelRequest {
    pub id: RequestId,
    pub start: Option<String>,
}

impl CancelRequest {
    pub fn new(start: Option<String>) -> Self {
        Self {
            id: RequestId::new(),
            start,
        }
    }
}

/// A response to an internal request to cancel execution of a document
#[derive(Debug)]
pub struct CancelResponse {
    pub id: RequestId,
}

impl CancelResponse {
    pub fn null() -> Self {
        Self {
            id: RequestId("".into()),
        }
    }

    pub fn new(id: RequestId) -> Self {
        Self { id }
    }
}
