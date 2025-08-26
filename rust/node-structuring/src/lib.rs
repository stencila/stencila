use common::smart_default::SmartDefault;
use schema::WalkNode;

use crate::{collector::Collector, replacer::Replacer, sectioner::Sectioner};

mod collector;
mod replacer;
mod sectioner;

#[cfg(test)]
mod tests;

/// Citation style options for in-text citations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CitationStyle {
    /// Author-year citations like (Smith, 2023)
    AuthorYear,

    /// Bracketed numeric citations like [1]
    BracketedNumeric,

    /// Parenthetic numeric citations like (1)
    ParentheticNumeric,

    /// Superscripted numeric citations like ¹
    SuperscriptedNumeric,
}

/// Options for document structuring
#[derive(Debug, Clone, SmartDefault)]
pub struct StructuringOptions {
    /// Whether to extract title from content
    #[default = true]
    pub extract_title: bool,

    /// Whether to create nested sections from headings
    #[default = true]
    pub sectioning: bool,

    /// Citation style to use for in-text citations.
    ///
    /// If None, will be determined automatically based on whether references
    /// are numbered and the relative frequency of detected styles within text.
    pub citation_style: Option<CitationStyle>,
}

/// Add structure to a document with default options
pub fn structuring<T: WalkNode>(node: &mut T) {
    structuring_with_options(node, StructuringOptions::default())
}

/// Add structure to a document with custom options
pub fn structuring_with_options<T: WalkNode>(node: &mut T, options: StructuringOptions) {
    let mut collector = Collector::new(options.clone());
    node.walk_mut(&mut collector);
    collector.determine_citation_style(options.citation_style);

    let mut replacer = Replacer::new(collector);
    node.walk_mut(&mut replacer);

    if options.sectioning {
        let mut sectioner = Sectioner::default();
        node.walk_mut(&mut sectioner);
    }
}
