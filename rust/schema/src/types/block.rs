// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::admonition::Admonition;
use super::call::Call;
use super::claim::Claim;
use super::code_block::CodeBlock;
use super::code_chunk::CodeChunk;
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
use super::section::Section;
use super::styled_block::StyledBlock;
use super::table::Table;
use super::thematic_break::ThematicBreak;

/// Union type in block content node types.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, SmartDefault, ReadNode)]
#[serde(untagged, crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum Block {
    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    Admonition(Admonition),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    Call(Call),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    Claim(Claim),

    CodeBlock(CodeBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    CodeChunk(CodeChunk),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    Figure(Figure),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    For(For),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    Form(Form),

    Heading(Heading),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    If(If),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    Include(Include),

    List(List),

    MathBlock(MathBlock),

    #[default]
    Paragraph(Paragraph),

    QuoteBlock(QuoteBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    Section(Section),

    StyledBlock(StyledBlock),

    Table(Table),

    ThematicBreak(ThematicBreak),
}
