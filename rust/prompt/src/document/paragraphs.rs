use crate::prelude::*;

/// A paragraph in the current document
#[derive(Debug, Default, Clone, Trace)]
#[rquickjs::class]
pub struct Paragraph {
    /// The Markdown content of the paragraph
    #[qjs(get, enumerable)]
    markdown: String,
}

impl Paragraph {
    #[cfg(test)]
    pub fn new(content: &str) -> Self {
        Self {
            markdown: content.to_string(),
        }
    }

    pub fn markdown(&self) -> String {
        self.markdown.clone()
    }
}

impl From<&schema::Paragraph> for Paragraph {
    fn from(paragraph: &schema::Paragraph) -> Self {
        Self {
            markdown: to_markdown(&paragraph.content),
        }
    }
}

#[rquickjs::methods]
impl Paragraph {
    #[qjs(rename = PredefinedAtom::ToJSON)]
    fn to_json<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>, Error> {
        let obj = Object::new(ctx)?;
        obj.set("markdown", self.markdown.clone())?;
        Ok(obj)
    }
}

/// The paragraphs in a document
#[derive(Debug, Default, Clone, Trace)]
#[rquickjs::class]
pub struct Paragraphs {
    /// Whether to ignore paragraphs that are pushed, entered and exited
    ///
    /// This can be used to ignore paragraphs such as those in footnotes,
    /// figure captions and table cells which will not normally be considered
    /// paragraphs in the main flow of the document.
    pub(super) ignore: bool,

    items: Vec<Paragraph>,
    cursor: Option<usize>,
    current: Option<usize>,
}

impl Paragraphs {
    /// Create a new list of paragraphs
    #[cfg(test)]
    pub(super) fn new(items: Vec<Paragraph>) -> Self {
        Self {
            items,
            ..Default::default()
        }
    }

    /// Push a paragraph onto the list
    pub fn push(&mut self, item: Paragraph) {
        if !self.ignore {
            self.items.push(item);
        }
    }
}

#[rquickjs::methods]
impl Paragraphs {
    /// Enter a paragraph
    #[qjs(rename = "_enter")]
    pub fn enter(&mut self) {
        if !self.ignore {
            self.cursor = self.cursor.map(|cursor| cursor + 1).or(Some(0));
            self.current = self.cursor;
        }
    }

    /// Exit a paragraph
    #[qjs(rename = "_exit")]
    pub fn exit(&mut self) {
        if !self.ignore {
            self.current = None;
        }
    }

    /// Get the count of all paragraphs
    #[qjs(get)]
    fn count(&self) -> usize {
        self.items.len()
    }

    /// Get all paragraphs
    #[qjs(get)]
    fn all(&self) -> Vec<Paragraph> {
        self.items.clone()
    }

    /// Get the first paragraph (if any)
    #[qjs(get)]
    fn first(&self) -> Option<Paragraph> {
        self.items.first().cloned()
    }

    /// Get the last paragraph (if any)
    #[qjs(get)]
    fn last(&self) -> Option<Paragraph> {
        self.items.last().cloned()
    }

    /// Get the previous paragraph (if any)
    #[qjs(get)]
    pub fn previous(&self) -> Option<Paragraph> {
        self.cursor.and_then(|cursor| {
            let index = if self.current.is_some() {
                // Currently in a paragraph
                if cursor == 0 {
                    // In first paragraph, so no previous
                    return None;
                } else {
                    cursor - 1
                }
            } else {
                // Not currently in a paragraph
                cursor
            };
            self.items.get(index).cloned()
        })
    }

    /// Get the current paragraph (if any)
    #[qjs(get)]
    fn current(&self) -> Option<Paragraph> {
        self.current
            .and_then(|current| self.items.get(current).cloned())
    }

    /// Get the next paragraph (if any)
    #[qjs(get)]
    pub fn next(&self) -> Option<Paragraph> {
        self.cursor
            .map(|cursor| self.items.get(cursor + 1).cloned())
            .unwrap_or_else(|| self.first())
    }
}
