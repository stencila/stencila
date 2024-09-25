use crate::prelude::*;

/// A table in the current document
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct Table {
    /// The label of the table
    #[qjs(get, enumerable)]
    pub label: Option<String>,

    // The caption as a Markdown string
    #[qjs(get, enumerable)]
    caption: Option<String>,

    /// The Markdown content of the table
    #[qjs(get, enumerable)]
    markdown: String,
}

impl Table {
    #[cfg(test)]
    pub fn new(caption: Option<&str>, markdown: &str) -> Self {
        Self {
            label: None,
            caption: caption.map(String::from),
            markdown: markdown.into(),
        }
    }
}

impl From<&schema::Table> for Table {
    fn from(table: &schema::Table) -> Self {
        Self {
            label: table.label.clone(),
            caption: table.caption.as_ref().map(to_markdown),
            markdown: to_markdown(table),
        }
    }
}

impl Table {
    pub fn markdown(&self) -> String {
        self.markdown.clone()
    }
}

#[rquickjs::methods]
impl Table {
    #[qjs(rename = PredefinedAtom::ToJSON)]
    fn to_json<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>, Error> {
        let obj = Object::new(ctx)?;
        obj.set("label", self.label.clone())?;
        obj.set("caption", self.caption.clone())?;
        obj.set("markdown", self.markdown.clone())?;
        Ok(obj)
    }
}

/// The tables in the current document
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct Tables {
    pub items: Vec<Table>,
    cursor: Option<usize>,
    current: Option<usize>,
}

impl Tables {
    /// Push a table onto the set
    pub(super) fn push(&mut self, item: Table) {
        self.items.push(item);
    }
}

#[rquickjs::methods]
impl Tables {
    /// Enter a table
    #[qjs(rename = "_enter")]
    pub(super) fn enter(&mut self) {
        self.cursor = self.cursor.map(|cursor| cursor + 1).or(Some(0));
        self.current = self.cursor;
    }

    /// Exit a table
    #[qjs(rename = "_exit")]
    pub(super) fn exit(&mut self) {
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
    pub fn previous(&self) -> Option<Table> {
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
    pub fn next(&self) -> Option<Table> {
        self.cursor
            .map(|cursor| self.items.get(cursor + 1).cloned())
            .unwrap_or_else(|| self.first())
    }
}
