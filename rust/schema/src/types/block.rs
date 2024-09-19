// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::admonition::Admonition;
use super::call_block::CallBlock;
use super::claim::Claim;
use super::code_block::CodeBlock;
use super::code_chunk::CodeChunk;
use super::delete_block::DeleteBlock;
use super::figure::Figure;
use super::for_block::ForBlock;
use super::form::Form;
use super::heading::Heading;
use super::if_block::IfBlock;
use super::include_block::IncludeBlock;
use super::insert_block::InsertBlock;
use super::instruction_block::InstructionBlock;
use super::list::List;
use super::math_block::MathBlock;
use super::modify_block::ModifyBlock;
use super::paragraph::Paragraph;
use super::quote_block::QuoteBlock;
use super::raw_block::RawBlock;
use super::replace_block::ReplaceBlock;
use super::section::Section;
use super::styled_block::StyledBlock;
use super::suggestion_block::SuggestionBlock;
use super::table::Table;
use super::thematic_break::ThematicBreak;

/// Union type in block content node types.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum Block {
    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    Admonition(Admonition),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    CallBlock(CallBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    Claim(Claim),

    CodeBlock(CodeBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    CodeChunk(CodeChunk),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    DeleteBlock(DeleteBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    Figure(Figure),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    ForBlock(ForBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    Form(Form),

    Heading(Heading),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    IfBlock(IfBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    IncludeBlock(IncludeBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    InsertBlock(InsertBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    InstructionBlock(InstructionBlock),

    List(List),

    MathBlock(MathBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    ModifyBlock(ModifyBlock),

    #[default]
    Paragraph(Paragraph),

    QuoteBlock(QuoteBlock),

    RawBlock(RawBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    ReplaceBlock(ReplaceBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    Section(Section),

    StyledBlock(StyledBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    SuggestionBlock(SuggestionBlock),

    Table(Table),

    ThematicBreak(ThematicBreak),
}
