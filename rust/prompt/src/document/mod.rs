use crate::prelude::*;

mod code_chunks;
mod figures;
mod headings;
mod math_blocks;
mod metadata;
mod node;
mod paragraphs;
mod sections;
mod tables;

#[cfg(test)]
mod tests;

use code_chunks::CodeChunks;
use figures::{Figure, Figures};
use headings::Headings;
use math_blocks::MathBlocks;
use metadata::Metadata;
use paragraphs::Paragraphs;
use sections::Sections;
use tables::{Table, Tables};

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

    /// Math blocks in the current document
    #[qjs(get, enumerable)]
    pub math_blocks: MathBlocks,
}

impl Document {
    /// Push a table onto the content and ignore paragraphs within it
    pub fn begin_table(&mut self, table: Table) {
        self.tables.push(table);
        self.paragraphs.ignore = true;
    }

    /// End ignoring paragraphs within a table
    pub fn end_table(&mut self) {
        self.paragraphs.ignore = false;
    }

    /// Enter a table and ignore any paragraphs within it
    pub fn enter_table(&mut self) {
        self.tables.enter();
        self.paragraphs.ignore = true;
    }

    /// Exit a table and end ignoring paragraphs within it
    pub fn exit_table(&mut self) {
        self.tables.exit();
        self.paragraphs.ignore = false;
    }

    /// Push a figure onto the content and ignore paragraphs within it
    pub fn begin_figure(&mut self, figure: Figure) {
        self.figures.push(figure);
        self.paragraphs.ignore = true;
    }

    /// End ignoring paragraphs within a figure
    pub fn end_figure(&mut self) {
        self.paragraphs.ignore = false;
    }

    /// Enter a figure and ignore any paragraphs within it
    pub fn enter_figure(&mut self) {
        self.figures.enter();
        self.paragraphs.ignore = true;
    }

    /// Exit a figure and end ignoring paragraphs within it
    pub fn exit_figure(&mut self) {
        self.figures.exit();
        self.paragraphs.ignore = false;
    }
}
