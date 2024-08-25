use crate::prelude::*;

/// A math block in the current document
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct MathBlock {
    /// The language of the math block
    #[qjs(get, enumerable)]
    language: Option<String>,

    /// The Markdown representation of the math block
    #[qjs(get, enumerable)]
    markdown: String,
}

impl From<&schema::MathBlock> for MathBlock {
    fn from(math_block: &schema::MathBlock) -> Self {
        Self {
            language: math_block.math_language.clone(),
            markdown: to_markdown(math_block),
        }
    }
}

#[rquickjs::methods]
impl MathBlock {
    #[qjs(rename = PredefinedAtom::ToJSON)]
    fn to_json<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>, Error> {
        let obj = Object::new(ctx)?;
        obj.set("language", self.language.clone())?;
        obj.set("markdown", self.markdown.clone())?;
        Ok(obj)
    }
}

/// The math blocks in the current document
/// 
/// Note that unlike other context collections in this crate
/// (e.g. CodeChunks), this collection does not have the notion
/// of the `current` math block since there are no nested nodes
/// in a math block (just strings).
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct MathBlocks {
    items: Vec<MathBlock>,
    cursor: Option<usize>,
}

impl MathBlocks {
    /// Push a math block onto the list
    pub fn push(&mut self, item: MathBlock) {
        self.items.push(item);
    }
}

#[rquickjs::methods]
impl MathBlocks {
    /// Step over a math block
    #[qjs(rename = "_step")]
    pub fn step(&mut self) {
        self.cursor = self.cursor.map(|cursor| cursor + 1).or(Some(0));
    }

    /// Get the count of all math blocks
    #[qjs(get)]
    fn count(&self) -> usize {
        self.items.len()
    }

    /// Get all math blocks
    #[qjs(get)]
    fn all(&self) -> Vec<MathBlock> {
        self.items.clone()
    }

    /// Get the first math block (if any)
    #[qjs(get)]
    fn first(&self) -> Option<MathBlock> {
        self.items.first().cloned()
    }

    /// Get the last math block (if any)
    #[qjs(get)]
    fn last(&self) -> Option<MathBlock> {
        self.items.last().cloned()
    }

    /// Get the preceding math blocks (if any)
    #[qjs(get)]
    fn preceding(&self) -> Vec<MathBlock> {
        let Some(cursor) = self.cursor else {
            return Vec::new();
        };

        self.items.iter().take(cursor + 1).cloned().collect()
    }

    /// Get the previous math block (if any)
    #[qjs(get)]
    fn previous(&self) -> Option<MathBlock> {
        self.cursor
            .and_then(|cursor| self.items.get(cursor).cloned())
    }

    /// Get the next math block (if any)
    #[qjs(get)]
    fn next(&self) -> Option<MathBlock> {
        self.cursor
            .map(|cursor| self.items.get(cursor + 1).cloned())
            .unwrap_or_else(|| self.first())
    }
}
