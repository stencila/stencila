// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::annotation::Annotation;
use super::audio_object::AudioObject;
use super::boolean::Boolean;
use super::button::Button;
use super::citation::Citation;
use super::citation_group::CitationGroup;
use super::code_expression::CodeExpression;
use super::code_inline::CodeInline;
use super::date::Date;
use super::date_time::DateTime;
use super::duration::Duration;
use super::emphasis::Emphasis;
use super::image_object::ImageObject;
use super::instruction_inline::InstructionInline;
use super::integer::Integer;
use super::link::Link;
use super::math_inline::MathInline;
use super::media_object::MediaObject;
use super::note::Note;
use super::null::Null;
use super::number::Number;
use super::parameter::Parameter;
use super::quote_inline::QuoteInline;
use super::sentence::Sentence;
use super::strikeout::Strikeout;
use super::strong::Strong;
use super::styled_inline::StyledInline;
use super::subscript::Subscript;
use super::suggestion_inline::SuggestionInline;
use super::superscript::Superscript;
use super::text::Text;
use super::time::Time;
use super::timestamp::Timestamp;
use super::underline::Underline;
use super::unsigned_integer::UnsignedInteger;
use super::video_object::VideoObject;

/// Union type for valid inline content.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum Inline {
    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    Annotation(Annotation),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    AudioObject(AudioObject),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    Button(Button),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    Citation(Citation),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    CitationGroup(CitationGroup),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    CodeExpression(CodeExpression),

    CodeInline(CodeInline),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    Date(Date),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    DateTime(DateTime),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    Duration(Duration),

    Emphasis(Emphasis),

    ImageObject(ImageObject),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    InstructionInline(InstructionInline),

    Link(Link),

    MathInline(MathInline),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    MediaObject(MediaObject),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    Note(Note),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    Parameter(Parameter),

    QuoteInline(QuoteInline),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    Sentence(Sentence),

    StyledInline(StyledInline),

    Strikeout(Strikeout),

    Strong(Strong),

    Subscript(Subscript),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    SuggestionInline(SuggestionInline),

    Superscript(Superscript),

    #[default]
    Text(Text),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    Time(Time),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    Timestamp(Timestamp),

    Underline(Underline),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    VideoObject(VideoObject),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(value = r#"Inline::Null(Null)"#))]
    Null(Null),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"Boolean::arbitrary().prop_map(Inline::Boolean)"#))]
    Boolean(Boolean),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"Integer::arbitrary().prop_map(Inline::Integer)"#))]
    Integer(Integer),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    UnsignedInteger(UnsignedInteger),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(value = r#"Inline::Number(1.23)"#))]
    Number(Number),
}
