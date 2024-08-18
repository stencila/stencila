use rquickjs::class::Trace;
pub(super) mod headings;
pub(super) mod metadata;
pub(super) mod paragraphs;

#[cfg(test)]
mod tests;

use headings::Headings;
use metadata::Metadata;
use paragraphs::Paragraphs;

/// The context of the current document
///
/// This intentionally condenses the rich, nested, structure of the document
/// into a flat structure that is more easily accessible from within prompts.
#[derive(Default, Trace)]
#[rquickjs::class]
pub struct Document {
    /// Metadata of the current document
    #[qjs(get)]
    pub metadata: Metadata,

    /// Document headings
    #[qjs(get)]
    pub headings: Headings,

    /// Document paragraphs
    #[qjs(get)]
    pub paragraphs: Paragraphs,
}
