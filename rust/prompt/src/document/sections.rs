use crate::prelude::*;

// Currently a section is just a string of the section type
type Section = String;

/// The sections in a document
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct Sections {
    items: Vec<Section>,
    cursor: Option<usize>,
    current: Option<usize>,
}

impl Sections {
    /// Create a new list of sections
    #[cfg(test)]
    pub(super) fn new(items: Vec<String>) -> Self {
        Self {
            items,
            cursor: None,
            current: None,
        }
    }

    /// Push a section onto the list
    pub fn push(&mut self, section: &schema::Section) {
        let section = section
            .section_type
            .as_ref()
            .map(|section_type| section_type.to_string())
            .unwrap_or_else(|| "Section".to_string());
        self.items.push(section);
    }

    /// Push a heading onto the list if its content matches that of one of the section types
    ///
    /// This allows us to treat [`schema::Heading`]s as section dividers without requiring.
    /// authors to use the more heavy weight section syntax.
    pub fn push_heading(&mut self, heading: &schema::Heading) {
        if let Some(section_type) = Self::heading_section_type(heading) {
            self.items.push(section_type.to_string());
        }
    }

    /// Enter a section defined by a heading
    pub fn enter_heading(&mut self, heading: &schema::Heading) {
        if Self::heading_section_type(heading).is_some() {
            self.enter();
        }
    }

    /// Returns a [`schema::SectionType`] if the heading if level one and it's content
    /// matches one of the section types
    fn heading_section_type(heading: &schema::Heading) -> Option<schema::SectionType> {
        if heading.level != 1 {
            return None;
        }

        let content = to_markdown(&heading.content).to_lowercase();

        use schema::SectionType::*;
        Some(match content.trim() {
            "abstract" => Abstract,
            "summary" => Summary,
            "introduction" => Introduction,
            "methods" | "materials and methods" => Methods,
            "materials" => Materials,
            "cases" => Cases,
            "results" => Results,
            "discussion" => Discussion,
            "supplementary materials" => SupplementaryMaterials,
            _ => return None,
        })
    }
}

#[rquickjs::methods]
impl Sections {
    /// Enter a section
    #[qjs(rename = "_enter")]
    pub fn enter(&mut self) {
        self.cursor = self.cursor.map(|cursor| cursor + 1).or(Some(0));
        self.current = self.cursor;
    }

    /// Exit a section
    #[qjs(rename = "_exit")]
    pub fn exit(&mut self) {
        self.current = None;
    }

    /// Get the count of all sections
    #[qjs(get)]
    fn count(&self) -> usize {
        self.items.len()
    }

    /// Get all sections
    #[qjs(get)]
    fn all(&self) -> Vec<Section> {
        self.items.clone()
    }

    /// Get the first section (if any)
    #[qjs(get)]
    fn first(&self) -> Option<Section> {
        self.items.first().cloned()
    }

    /// Get the last section (if any)
    #[qjs(get)]
    fn last(&self) -> Option<Section> {
        self.items.last().cloned()
    }

    /// Get the previous section (if any)
    #[qjs(get)]
    fn previous(&self) -> Option<Section> {
        self.cursor.and_then(|cursor| {
            let index = if self.current.is_some() {
                // Currently in a section
                if cursor == 0 {
                    // In first section, so no previous
                    return None;
                } else {
                    cursor - 1
                }
            } else {
                // Not currently in a section
                cursor
            };
            self.items.get(index).cloned()
        })
    }

    /// Get the current section (if any)
    #[qjs(get)]
    fn current(&self) -> Option<Section> {
        self.current
            .and_then(|current| self.items.get(current).cloned())
    }

    /// Get the next section (if any)
    #[qjs(get)]
    fn next(&self) -> Option<Section> {
        self.cursor
            .map(|cursor| self.items.get(cursor + 1).cloned())
            .unwrap_or_else(|| self.first())
    }
}
