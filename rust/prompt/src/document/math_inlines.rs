use crate::prelude::*;

/// An inline math node in the current document
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct MathInline {
    /// The language of the inline math
    #[qjs(get, enumerable)]
    language: Option<String>,

    /// The Markdown representation of the inline math
    #[qjs(get, enumerable)]
    markdown: String,
}

impl From<&schema::MathInline> for MathInline {
    fn from(math_block: &schema::MathInline) -> Self {
        Self {
            language: math_block.math_language.clone(),
            markdown: to_markdown(math_block),
        }
    }
}

#[rquickjs::methods]
impl MathInline {
    #[qjs(rename = PredefinedAtom::ToJSON)]
    fn to_json<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>, Error> {
        let obj = Object::new(ctx)?;
        obj.set("language", self.language.clone())?;
        obj.set("markdown", self.markdown.clone())?;
        Ok(obj)
    }
}

/// Inline math nodes in the current document
/// 
/// Note that unlike other context collections in this crate
/// (e.g. CodeChunks), this collection does not have the notion
/// of the `current` inline math since there are no nested nodes
/// in a inline math (just strings).
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct MathInlines {
    items: Vec<MathInline>,
    cursor: Option<usize>,
}

impl MathInlines {
    /// Push a inline math onto the list
    pub fn push(&mut self, item: MathInline) {
        self.items.push(item);
    }
}

#[rquickjs::methods]
impl MathInlines {
    /// Step over a inline math
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
    fn all(&self) -> Vec<MathInline> {
        self.items.clone()
    }

    /// Get the first inline math (if any)
    #[qjs(get)]
    fn first(&self) -> Option<MathInline> {
        self.items.first().cloned()
    }

    /// Get the last inline math (if any)
    #[qjs(get)]
    fn last(&self) -> Option<MathInline> {
        self.items.last().cloned()
    }

    /// Get the preceding math blocks (if any)
    #[qjs(get)]
    fn preceding(&self) -> Vec<MathInline> {
        let Some(cursor) = self.cursor else {
            return Vec::new();
        };

        self.items.iter().take(cursor + 1).cloned().collect()
    }

    /// Get the previous inline math (if any)
    #[qjs(get)]
    fn previous(&self) -> Option<MathInline> {
        self.cursor
            .and_then(|cursor| self.items.get(cursor).cloned())
    }

    /// Get the next inline math (if any)
    #[qjs(get)]
    fn next(&self) -> Option<MathInline> {
        self.cursor
            .map(|cursor| self.items.get(cursor + 1).cloned())
            .unwrap_or_else(|| self.first())
    }
}
