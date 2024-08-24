use crate::prelude::*;

/// A table in the current document
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct Table {
    // The caption as a Markdown string
    #[qjs(get, enumerable)]
    caption: Option<String>,

    /// The Markdown content of the table
    #[qjs(get, enumerable)]
    content: String,
}

impl Table {
    #[cfg(test)]
    pub fn new(caption: Option<&str>, content: &str) -> Self {
        Self {
            caption: caption.map(String::from),
            content: content.into(),
        }
    }
}

impl From<&schema::Table> for Table {
    fn from(table: &schema::Table) -> Self {
        Self {
            caption: table.caption.as_ref().map(|caption| to_markdown(caption)),
            content: to_markdown(&table.rows),
        }
    }
}

#[rquickjs::methods]
impl Table {
    #[qjs(rename = PredefinedAtom::ToJSON)]
    fn to_json<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>, Error> {
        let obj = Object::new(ctx)?;
        obj.set("caption", self.caption.clone())?;
        obj.set("content", self.content.clone())?;
        Ok(obj)
    }
}

/// The tables in the current document
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct Tables {
    items: Vec<Table>,
    cursor: Option<usize>,
    current: Option<usize>,
}

impl Tables {
    /// Create a new set of tables
    pub fn new(items: Vec<Table>) -> Self {
        Self {
            items,
            cursor: None,
            current: None,
        }
    }

    /// Push a table onto the set
    pub fn push(&mut self, item: Table) {
        self.items.push(item);
    }
}

#[rquickjs::methods]
impl Tables {
    /// Enter a table
    #[qjs(rename = "_enter")]
    pub fn enter(&mut self) {
        self.cursor = self.cursor.map(|cursor| cursor + 1).or(Some(0));
        self.current = self.cursor.clone();
    }

    /// Exit a table
    #[qjs(rename = "_exit")]
    pub fn exit(&mut self) {
        self.current = None;
    }

    /// Get the count of all tables
    #[qjs(get)]
    fn count(&self) -> usize {
        self.items.len()
    }

    /// Get all tables
    #[qjs(get)]
    fn all(&self) -> Vec<Table> {
        self.items.clone()
    }

    /// Get the first table (if any)
    #[qjs(get)]
    fn first(&self) -> Option<Table> {
        self.items.first().cloned()
    }

    /// Get the last table (if any)
    #[qjs(get)]
    fn last(&self) -> Option<Table> {
        self.items.last().cloned()
    }

    /// Get the previous table (if any)
    #[qjs(get)]
    fn previous(&self) -> Option<Table> {
        self.cursor.and_then(|cursor| {
            let index = if self.current.is_some() {
                // Currently in a table
                if cursor == 0 {
                    // In first table, so no previous
                    return None;
                } else {
                    cursor - 1
                }
            } else {
                // Not currently in a table
                cursor
            };
            self.items.get(index).cloned()
        })
    }

    /// Get the current table (if any)
    #[qjs(get)]
    fn current(&self) -> Option<Table> {
        self.current
            .and_then(|current| self.items.get(current).cloned())
    }

    /// Get the next table (if any)
    #[qjs(get)]
    fn next(&self) -> Option<Table> {
        self.cursor
            .map(|cursor| self.items.get(cursor + 1).cloned())
            .unwrap_or_else(|| self.first())
    }
}
