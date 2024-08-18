use rquickjs::class::Trace;

use codec_markdown_trait::to_markdown;
use kernel::schema;

/// The headings in a document
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct Headings {
    items: Vec<Heading>,
    cursor: Option<usize>,
}

impl Headings {
    /// Create a new list of headings
    pub fn new(items: Vec<Heading>) -> Self {
        Self {
            items,
            cursor: None,
        }
    }

    /// Push a heading onto the list
    pub fn push(&mut self, item: Heading) {
        self.items.push(item);
    }
}

#[rquickjs::methods]
impl Headings {
    /// Move the heading cursor forward
    #[qjs()]
    fn _forward(&mut self) {
        self.cursor = self.cursor.map(|cursor| cursor + 1).or(Some(0));
    }

    /// Get all headings
    #[qjs()]
    fn all(&self) -> Vec<Heading> {
        self.items.clone()
    }

    /// Get the first heading (if any)
    #[qjs()]
    fn first(&self) -> Option<Heading> {
        self.items.first().cloned()
    }

    /// Get the last heading (if any)
    #[qjs()]
    fn last(&self) -> Option<Heading> {
        self.items.last().cloned()
    }

    /// Get the previous heading (if any)
    #[qjs()]
    fn previous(&self) -> Option<Heading> {
        self.cursor.and_then(|cursor| {
            if cursor == 0 {
                None
            } else {
                self.items.get(cursor - 1).cloned()
            }
        })
    }

    /// Get the current heading (if any)
    #[qjs()]
    fn current(&self) -> Option<Heading> {
        self.cursor
            .and_then(|cursor| self.items.get(cursor).cloned())
    }

    /// Get the next heading (if any)
    #[qjs()]
    fn next(&self) -> Option<Heading> {
        match self.cursor {
            Some(cursor) => self.items.get(cursor + 1).cloned(),
            None => self.first(),
        }
    }

    /// Get the current hierarchy of headings
    #[qjs()]
    fn hierarchy(&self) -> Vec<Heading> {
        let mut headings = Vec::new();

        if let Some(mut cursor) = self.cursor {
            let mut level = i32::MAX;
            loop {
                let heading = &self.items[cursor];
                if heading.level < level {
                    headings.push(heading.clone());
                    if heading.level == 1 {
                        break;
                    } else {
                        level = heading.level;
                    }
                }
                if cursor == 0 {
                    break;
                }
                cursor -= 1;
            }
        }

        headings.reverse();
        headings
    }
}

/// A heading in the current document
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct Heading {
    // The level of the document
    #[qjs(get)]
    level: i32,

    /// The Markdown content of the document
    #[qjs(get)]
    content: String,
}

impl Heading {
    pub fn new(level: i32, content: &str) -> Self {
        Self {
            level,
            content: content.to_string(),
        }
    }
}

impl From<&schema::Heading> for Heading {
    fn from(heading: &schema::Heading) -> Self {
        Self {
            level: heading.level as i32,
            content: to_markdown(&heading.content),
        }
    }
}
