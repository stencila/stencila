use crate::prelude::*;

mod code_chunks;
mod figures;
mod headings;
mod math_blocks;
mod math_inlines;
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
use math_inlines::MathInlines;
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

    /// Inline math in the current document
    #[qjs(get, enumerable)]
    pub math_inlines: MathInlines,
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

    /// Get the previous block as Markdown
    ///
    /// Because this context does not keep track of the all blocks
    /// as a collection (yet!), this method takes the approach of
    /// returning the more complex node types first
    pub fn previous_block(&self) -> String {
        if let Some(previous) = self.code_chunks.previous() {
            return previous.markdown_with_outputs();
        }

        if let Some(previous) = self.figures.previous() {
            return previous.markdown();
        }

        if let Some(previous) = self.tables.previous() {
            return previous.markdown();
        }

        if let Some(previous) = self.math_blocks.previous() {
            return previous.markdown();
        }

        if let Some(previous) = self.paragraphs.previous() {
            return previous.markdown();
        }

        if let Some(previous) = self.headings.previous() {
            return previous.markdown();
        }

        String::new()
    }

    /// Get the next block as Markdown
    pub fn next_block(&self) -> String {
        if let Some(next) = self.code_chunks.next() {
            return next.markdown_with_outputs();
        }

        if let Some(next) = self.figures.next() {
            return next.markdown();
        }

        if let Some(next) = self.tables.next() {
            return next.markdown();
        }

        if let Some(next) = self.math_blocks.next() {
            return next.markdown();
        }

        if let Some(next) = self.paragraphs.next() {
            return next.markdown();
        }

        if let Some(next) = self.headings.next() {
            return next.markdown();
        }

        String::new()
    }
}
