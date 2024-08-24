use crate::prelude::*;

mod code_chunks;
mod figures;
mod headings;
mod metadata;
mod node;
mod paragraphs;
mod sections;
mod tables;

#[cfg(test)]
mod tests;

use code_chunks::CodeChunks;
use figures::Figures;
use headings::Headings;
use metadata::Metadata;
use paragraphs::Paragraphs;
use sections::Sections;
use tables::Tables;

/// The context of the current document
///
/// This intentionally condenses the rich, nested, structure of the document
/// into a flat structure that is more easily accessible from within prompts.
#[derive(Default, Clone, Trace)]
#[rquickjs::class(rename_all = "camelCase")]
pub struct Document {
    /// Metadata of the current document
    #[qjs(get, enumerable)]
    pub metadata: Metadata,

    /// Sections in the current document
    #[qjs(get, enumerable)]
    pub sections: Sections,

    /// Headings in the current document
    #[qjs(get, enumerable)]
    pub headings: Headings,

    /// Paragraphs in the current document
    #[qjs(get, enumerable)]
    pub paragraphs: Paragraphs,

    /// Tables in the current document
    #[qjs(get, enumerable)]
    pub tables: Tables,

    /// Figures in the current document
    #[qjs(get, enumerable)]
    pub figures: Figures,

    /// Code chunks in the current document
    #[qjs(get, enumerable)]
    pub code_chunks: CodeChunks,
}
