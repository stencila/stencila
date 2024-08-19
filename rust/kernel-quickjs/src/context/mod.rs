mod document;
mod kernels;

/// The execution context for a prompt
#[derive(Default)]
pub struct Context {
    /// The current document
    pub document: document::Document,

    /// The execution kernels associated with the document
    pub kernels: kernels::Kernels
}

