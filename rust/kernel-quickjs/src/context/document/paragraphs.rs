use rquickjs::class::Trace;

use codec_markdown_trait::to_markdown;
use kernel::schema;

/// The paragraphs in a document
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct Paragraphs {
    items: Vec<Paragraph>,
    cursor: Option<usize>,
}

impl Paragraphs {
    /// Create a new list of paragraphs
    pub fn new(items: Vec<Paragraph>) -> Self {
        Self {
            items,
            cursor: None,
        }
    }

    /// Push a paragraph onto the list
    pub fn push(&mut self, item: Paragraph) {
        self.items.push(item);
    }
}

#[rquickjs::methods]
impl Paragraphs {
    /// Move the paragraph cursor forward
    #[qjs()]
    fn _forward(&mut self) {
        self.cursor = self.cursor.map(|cursor| cursor + 1).or(Some(0));
    }

    /// Get all paragraphs
    #[qjs()]
    fn all(&self) -> Vec<Paragraph> {
        self.items.clone()
    }

    /// Get the first paragraph (if any)
    #[qjs()]
    fn first(&self) -> Option<Paragraph> {
        self.items.first().cloned()
    }

    /// Get the last paragraph (if any)
    #[qjs()]
    fn last(&self) -> Option<Paragraph> {
        self.items.last().cloned()
    }

    /// Get the previous paragraph (if any)
    #[qjs()]
    fn previous(&self) -> Option<Paragraph> {
        self.cursor.and_then(|cursor| {
            if cursor == 0 {
                None
            } else {
                self.items.get(cursor - 1).cloned()
            }
        })
    }

    /// Get the current paragraph (if any)
    #[qjs()]
    fn current(&self) -> Option<Paragraph> {
        self.cursor
            .and_then(|cursor| self.items.get(cursor).cloned())
    }

    /// Get the next paragraph (if any)
    #[qjs()]
    fn next(&self) -> Option<Paragraph> {
        match self.cursor {
            Some(cursor) => self.items.get(cursor + 1).cloned(),
            None => self.first(),
        }
    }
}

/// A paragraph in the current document
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct Paragraph {
    /// The Markdown content of the paragraph
    #[qjs(get, enumerable)]
    content: String,
}

impl Paragraph {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
        }
    }
}

impl From<&schema::Paragraph> for Paragraph {
    fn from(paragraph: &schema::Paragraph) -> Self {
        Self {
            content: to_markdown(&paragraph.content),
        }
    }
}
