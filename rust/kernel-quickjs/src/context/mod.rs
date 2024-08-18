mod document;

/// The execution context for a prompt
#[derive(Default)]
pub struct Context {
    /// A representation of the current document
    pub document: document::Document,
}

