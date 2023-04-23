use crate::prelude::*;

use super::call::Call;
use super::claim::Claim;
use super::code_block::CodeBlock;
use super::code_chunk::CodeChunk;
use super::division::Division;
use super::figure::Figure;
use super::r#for::For;
use super::form::Form;
use super::heading::Heading;
use super::r#if::If;
use super::include::Include;
use super::list::List;
use super::math_block::MathBlock;
use super::paragraph::Paragraph;
use super::quote_block::QuoteBlock;
use super::table::Table;
use super::thematic_break::ThematicBreak;

/// Union type for block content node types.
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Defaults, Strip, Read, Write, ToHtml)]
#[serde(untagged, crate = "common::serde")]
#[def = "Paragraph(Paragraph::default())"]
pub enum Block {
    Call(Call),
    Claim(Claim),
    CodeBlock(CodeBlock),
    CodeChunk(CodeChunk),
    Division(Division),
    Figure(Figure),
    For(For),
    Form(Form),
    Heading(Heading),
    If(If),
    Include(Include),
    List(List),
    MathBlock(MathBlock),
    Paragraph(Paragraph),
    QuoteBlock(QuoteBlock),
    Table(Table),
    ThematicBreak(ThematicBreak),
}
