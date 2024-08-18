use rquickjs::class::Trace;

pub(super) mod code_chunks;
pub(super) mod headings;
pub(super) mod metadata;
pub(super) mod node;
pub(super) mod paragraphs;

#[cfg(test)]
mod tests;

use code_chunks::CodeChunks;
use headings::Headings;
use metadata::Metadata;
use paragraphs::Paragraphs;

/// The context of the current document
///
/// This intentionally condenses the rich, nested, structure of the document
/// into a flat structure that is more easily accessible from within prompts.
#[derive(Default, Trace)]
#[rquickjs::class(rename_all = "camelCase")]
pub struct Document {
    /// Metadata of the current document
    #[qjs(get, enumerable)]
    pub metadata: Metadata,

    /// Document headings
    #[qjs(get, enumerable)]
    pub headings: Headings,

    /// Document paragraphs
    #[qjs(get, enumerable)]
    pub paragraphs: Paragraphs,

    /// Document code chunks
    #[qjs(get, enumerable)]
    pub code_chunks: CodeChunks,
}
