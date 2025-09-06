// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::admonition::Admonition;
use super::appendix_break::AppendixBreak;
use super::audio_object::AudioObject;
use super::call_block::CallBlock;
use super::chat::Chat;
use super::chat_message::ChatMessage;
use super::chat_message_group::ChatMessageGroup;
use super::claim::Claim;
use super::code_block::CodeBlock;
use super::code_chunk::CodeChunk;
use super::datatable::Datatable;
use super::excerpt::Excerpt;
use super::figure::Figure;
use super::file::File;
use super::for_block::ForBlock;
use super::form::Form;
use super::heading::Heading;
use super::if_block::IfBlock;
use super::image_object::ImageObject;
use super::include_block::IncludeBlock;
use super::inlines_block::InlinesBlock;
use super::instruction_block::InstructionBlock;
use super::island::Island;
use super::list::List;
use super::math_block::MathBlock;
use super::paragraph::Paragraph;
use super::prompt_block::PromptBlock;
use super::quote_block::QuoteBlock;
use super::raw_block::RawBlock;
use super::section::Section;
use super::styled_block::StyledBlock;
use super::suggestion_block::SuggestionBlock;
use super::supplement::Supplement;
use super::table::Table;
use super::thematic_break::ThematicBreak;
use super::video_object::VideoObject;
use super::walkthrough::Walkthrough;

/// Union type in block content node types.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(untagged)]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum Block {
    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    Admonition(Admonition),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    AppendixBreak(AppendixBreak),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    AudioObject(AudioObject),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    CallBlock(CallBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    Chat(Chat),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    ChatMessage(ChatMessage),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    ChatMessageGroup(ChatMessageGroup),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    Claim(Claim),

    CodeBlock(CodeBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    CodeChunk(CodeChunk),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    Datatable(Datatable),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    Excerpt(Excerpt),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    Figure(Figure),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    File(File),

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
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    ImageObject(ImageObject),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    IncludeBlock(IncludeBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    InlinesBlock(InlinesBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    InstructionBlock(InstructionBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    Island(Island),

    List(List),

    MathBlock(MathBlock),

    #[default]
    Paragraph(Paragraph),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    PromptBlock(PromptBlock),

    QuoteBlock(QuoteBlock),

    RawBlock(RawBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    Section(Section),

    StyledBlock(StyledBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    SuggestionBlock(SuggestionBlock),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    Supplement(Supplement),

    Table(Table),

    ThematicBreak(ThematicBreak),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    VideoObject(VideoObject),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    Walkthrough(Walkthrough),
}
