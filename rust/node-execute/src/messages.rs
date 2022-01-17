use node_patch::Patch;

pub struct PatchMessage {
    pub patch: Patch,
    pub compile: bool,
    pub execute: bool,
}

pub struct CompileMessage {
    pub execute: bool,
    pub from: Option<String>,
}

pub struct ExecuteMessage {
    pub from: Option<String>,
}
