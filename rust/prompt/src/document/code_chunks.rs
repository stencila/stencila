use kernel_quickjs::kernel::common::itertools::Itertools;

use crate::prelude::*;

use super::node::Node;

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

    /// The label type of the code chunk
    #[qjs(get, enumerable)]
    pub label_type: Option<String>,

    /// The label of the code chunk
    #[qjs(get, enumerable)]
    pub label: Option<String>,

    /// The Markdown representation of the code chunk (without outputs)
    #[qjs(get, enumerable)]
    markdown: String,
}

impl CodeChunk {
    #[cfg(test)]
    pub fn new(language: &str, code: &str, outputs: Option<Vec<Node>>) -> Self {
        Self {
            language: (!language.is_empty()).then(|| language.into()),
            code: code.into(),
            outputs,
            label_type: None,
            label: None,
            markdown: String::new(),
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
            label_type: code_chunk.label_type.as_ref().map(|lt| lt.to_string()),
            label: code_chunk.label.clone(),
            markdown: to_markdown(code_chunk),
        }
    }
}

impl CodeChunk {
    pub fn markdown(&self) -> String {
        self.markdown.clone()
    }

    pub fn markdown_with_outputs(&self) -> String {
        let outputs = self
            .outputs
            .iter()
            .flatten()
            .map(|node| &node.markdown)
            .join("\n\n");

        if outputs.is_empty() {
            self.markdown.clone()
        } else {
            [&self.markdown, "\n\nOutputs:\n\n", &outputs].concat()
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
        obj.set("label_type", self.label_type.clone())?;
        obj.set("label", self.label.clone())?;
        obj.set("markdown", self.markdown.clone())?;

        Ok(obj)
    }
}

/// The code chunks in a document
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct CodeChunks {
    pub items: Vec<CodeChunk>,
    cursor: Option<usize>,
    current: Option<usize>,
}

impl CodeChunks {
    /// Create a new list of code chunks
    #[cfg(test)]
    pub(super) fn new(items: Vec<CodeChunk>) -> Self {
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
        self.current = self.cursor;
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

    /// Get the preceding code chunks (if any)
    #[qjs(get)]
    fn preceding(&self) -> Vec<CodeChunk> {
        let Some(cursor) = self.cursor else {
            return Vec::new();
        };

        let take = if self.current.is_some() {
            cursor
        } else {
            cursor + 1
        };

        self.items.iter().take(take).cloned().collect()
    }

    /// Get the previous code chunk (if any)
    #[qjs(get)]
    pub fn previous(&self) -> Option<CodeChunk> {
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
    pub fn next(&self) -> Option<CodeChunk> {
        self.cursor
            .map(|cursor| self.items.get(cursor + 1).cloned())
            .unwrap_or_else(|| self.first())
    }
}
