// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::audio_object::AudioObject;
use super::boolean::Boolean;
use super::button::Button;
use super::cite::Cite;
use super::cite_group::CiteGroup;
use super::code_expression::CodeExpression;
use super::code_inline::CodeInline;
use super::date::Date;
use super::date_time::DateTime;
use super::delete_inline::DeleteInline;
use super::duration::Duration;
use super::emphasis::Emphasis;
use super::image_object::ImageObject;
use super::insert_inline::InsertInline;
use super::instruction_inline::InstructionInline;
use super::integer::Integer;
use super::link::Link;
use super::math_inline::MathInline;
use super::media_object::MediaObject;
use super::modify_inline::ModifyInline;
use super::note::Note;
use super::null::Null;
use super::number::Number;
use super::parameter::Parameter;
use super::quote_inline::QuoteInline;
use super::replace_inline::ReplaceInline;
use super::strikeout::Strikeout;
use super::strong::Strong;
use super::styled_inline::StyledInline;
use super::subscript::Subscript;
use super::superscript::Superscript;
use super::text::Text;
use super::time::Time;
use super::timestamp::Timestamp;
use super::underline::Underline;
use super::unsigned_integer::UnsignedInteger;
use super::video_object::VideoObject;

/// Union type for valid inline content.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum Inline {
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
    Cite(Cite),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    #[cfg_attr(feature = "proptest-max", proptest(skip))]
    CiteGroup(CiteGroup),

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
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    DeleteInline(DeleteInline),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    Duration(Duration),

    Emphasis(Emphasis),

    ImageObject(ImageObject),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    InsertInline(InsertInline),

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
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    ModifyInline(ModifyInline),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    Note(Note),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    Parameter(Parameter),

    QuoteInline(QuoteInline),

    #[cfg_attr(feature = "proptest-min", proptest(skip))]
    #[cfg_attr(feature = "proptest-low", proptest(skip))]
    #[cfg_attr(feature = "proptest-high", proptest(skip))]
    ReplaceInline(ReplaceInline),

    StyledInline(StyledInline),

    Strikeout(Strikeout),

    Strong(Strong),

    Subscript(Subscript),

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
