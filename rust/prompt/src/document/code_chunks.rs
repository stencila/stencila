use crate::prelude::*;

use super::node::Node;

/// The code chunks in a document
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct CodeChunks {
    items: Vec<CodeChunk>,
    cursor: Option<usize>,
    current: Option<usize>,
}

impl CodeChunks {
    /// Create a new list of code chunks
    pub fn new(items: Vec<CodeChunk>) -> Self {
        Self {
            items,
            cursor: None,
            current: None,
        }
    }

    /// Push a code chunk onto the list
    pub fn push(&mut self, item: CodeChunk) {
        self.items.push(item);
    }
}

#[rquickjs::methods]
impl CodeChunks {
    /// Enter a code chunk
    #[qjs(rename = "_enter")]
    pub fn enter(&mut self) {
        self.cursor = self.cursor.map(|cursor| cursor + 1).or(Some(0));
        self.current = self.cursor.clone();
    }

    /// Exit a code chunk
    #[qjs(rename = "_exit")]
    pub fn exit(&mut self) {
        self.current = None;
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
            let index = if self.current.is_some() {
                // Currently in a code chunk
                if cursor == 0 {
                    // In first code chunk, so no previous
                    return None;
                } else {
                    cursor - 1
                }
            } else {
                // Not currently in a code chunk
                cursor
            };
            self.items.get(index).cloned()
        })
    }

    /// Get the current code chunk (if any)
    #[qjs(get)]
    fn current(&self) -> Option<CodeChunk> {
        self.current
            .and_then(|current| self.items.get(current).cloned())
    }

    /// Get the next code chunk (if any)
    #[qjs(get)]
    fn next(&self) -> Option<CodeChunk> {
        self.cursor
            .map(|cursor| self.items.get(cursor + 1).cloned())
            .unwrap_or_else(|| self.first())
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

#[rquickjs::methods]
impl CodeChunk {
    #[qjs(rename = PredefinedAtom::ToJSON)]
    fn to_json<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>, Error> {
        let obj = Object::new(ctx)?;
        obj.set("language", self.language.clone())?;
        obj.set("code", self.code.clone())?;
        obj.set("outputs", self.outputs.clone())?;
        Ok(obj)
    }
}
