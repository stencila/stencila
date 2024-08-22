use crate::prelude::*;

use super::node::Node;

/// The code chunks in a document
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct CodeChunks {
    items: Vec<CodeChunk>,
    cursor: Option<usize>,
}

impl CodeChunks {
    /// Create a new list of code chunks
    pub fn new(items: Vec<CodeChunk>) -> Self {
        Self {
            items,
            cursor: None,
        }
    }

    /// Push a code chunk onto the list
    pub fn push(&mut self, item: CodeChunk) {
        self.items.push(item);
    }
}

#[rquickjs::methods]
impl CodeChunks {
    /// Move the code chunk cursor forward
    #[qjs(rename = "_forward")]
    pub fn forward(&mut self) {
        self.cursor = self.cursor.map(|cursor| cursor + 1).or(Some(0));
    }

    /// Get the count of all code chunks
    #[qjs(get)]
    fn count(&self) -> usize {
        self.items.len()
    }

    /// Get all code chunks
    #[qjs(get)]
    fn all(&self) -> Vec<CodeChunk> {
        self.items.clone()
    }

    /// Get the first code chunk (if any)
    #[qjs(get)]
    fn first(&self) -> Option<CodeChunk> {
        self.items.first().cloned()
    }

    /// Get the last code chunk (if any)
    #[qjs(get)]
    fn last(&self) -> Option<CodeChunk> {
        self.items.last().cloned()
    }

    /// Get the previous code chunk (if any)
    #[qjs(get)]
    fn previous(&self) -> Option<CodeChunk> {
        self.cursor.and_then(|cursor| {
            if cursor == 0 {
                None
            } else {
                self.items.get(cursor - 1).cloned()
            }
        })
    }

    /// Get the current code chunk (if any)
    #[qjs(get)]
    fn current(&self) -> Option<CodeChunk> {
        self.cursor
            .and_then(|cursor| self.items.get(cursor).cloned())
    }

    /// Get the next code chunk (if any)
    #[qjs(get)]
    fn next(&self) -> Option<CodeChunk> {
        match self.cursor {
            Some(cursor) => self.items.get(cursor + 1).cloned(),
            None => self.first(),
        }
    }
}

/// A code chunk in the current document
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct CodeChunk {
    /// The language of the code chunk
    #[qjs(get, enumerable)]
    language: Option<String>,

    /// The code of the code chunk
    #[qjs(get, enumerable)]
    code: String,

    /// The outputs of the code chunk
    #[qjs(get, enumerable)]
    outputs: Option<Vec<Node>>,
}

impl CodeChunk {
    #[cfg(test)]
    pub fn new(language: &str, code: &str, outputs: Option<Vec<Node>>) -> Self {
        Self {
            language: (!language.is_empty()).then(|| language.into()),
            code: code.into(),
            outputs,
        }
    }
}

impl From<&schema::CodeChunk> for CodeChunk {
    fn from(code_chunk: &schema::CodeChunk) -> Self {
        Self {
            language: code_chunk.programming_language.clone(),
            code: code_chunk.code.string.clone(),
            outputs: code_chunk
                .outputs
                .as_ref()
                .map(|outputs| outputs.iter().map(Node::from).collect()),
        }
    }
}
