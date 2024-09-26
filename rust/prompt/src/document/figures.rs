use crate::prelude::*;

/// A figure in the current document
///
/// Currently, only the figure caption is used.
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct Figure {
    /// The label of the figure
    #[qjs(get, enumerable)]
    pub label: Option<String>,

    // The caption as a Markdown string
    #[qjs(get, enumerable)]
    caption: Option<String>,
}

impl Figure {
    #[cfg(test)]
    pub fn new(caption: Option<&str>) -> Self {
        Self {
            label: None,
            caption: caption.map(String::from),
        }
    }
}

impl From<&schema::Figure> for Figure {
    fn from(figure: &schema::Figure) -> Self {
        Self {
            label: figure.label.clone(),
            caption: figure.caption.as_ref().map(to_markdown),
        }
    }
}

#[rquickjs::methods]
impl Figure {
    #[qjs()]
    pub fn markdown(&self) -> String {
        format!(
            "::: figure\n\n{}\n\n:::\n",
            self.caption.as_deref().unwrap_or_default()
        )
    }

    #[qjs(rename = PredefinedAtom::ToJSON)]
    fn to_json<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>, Error> {
        let obj = Object::new(ctx)?;
        obj.set("label", self.label.clone())?;
        obj.set("caption", self.caption.clone())?;
        Ok(obj)
    }
}

/// The figures in the current document
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct Figures {
    pub items: Vec<Figure>,
    cursor: Option<usize>,
    current: Option<usize>,
}

impl Figures {
    /// Push a figure onto the set
    pub(super) fn push(&mut self, item: Figure) {
        self.items.push(item);
    }
}

#[rquickjs::methods]
impl Figures {
    /// Enter a figure
    #[qjs(rename = "_enter")]
    pub(super) fn enter(&mut self) {
        self.cursor = self.cursor.map(|cursor| cursor + 1).or(Some(0));
        self.current = self.cursor;
    }

    /// Exit a figure
    #[qjs(rename = "_exit")]
    pub(super) fn exit(&mut self) {
        self.current = None;
    }

    /// Get the count of all figures
    #[qjs(get)]
    fn count(&self) -> usize {
        self.items.len()
    }

    /// Get all figures
    #[qjs(get)]
    fn all(&self) -> Vec<Figure> {
        self.items.clone()
    }

    /// Get the first figure (if any)
    #[qjs(get)]
    fn first(&self) -> Option<Figure> {
        self.items.first().cloned()
    }

    /// Get the last figure (if any)
    #[qjs(get)]
    fn last(&self) -> Option<Figure> {
        self.items.last().cloned()
    }

    /// Get the previous figure (if any)
    #[qjs(get)]
    pub fn previous(&self) -> Option<Figure> {
        self.cursor.and_then(|cursor| {
            let index = if self.current.is_some() {
                // Currently in a figure
                if cursor == 0 {
                    // In first figure, so no previous
                    return None;
                } else {
                    cursor - 1
                }
            } else {
                // Not currently in a figure
                cursor
            };
            self.items.get(index).cloned()
        })
    }

    /// Get the current figure (if any)
    #[qjs(get)]
    fn current(&self) -> Option<Figure> {
        self.current
            .and_then(|current| self.items.get(current).cloned())
    }

    /// Get the next figure (if any)
    #[qjs(get)]
    pub fn next(&self) -> Option<Figure> {
        self.cursor
            .map(|cursor| self.items.get(cursor + 1).cloned())
            .unwrap_or_else(|| self.first())
    }
}
