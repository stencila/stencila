use pandoc_types::definition::{self as pandoc, Attr};

use codec::schema::*;
use codec_text_trait::to_text;

use crate::shared::{attrs_empty, PandocDecodeContext, PandocEncodeContext};

pub(super) fn inlines_to_pandoc(
    inlines: &[Inline],
    context: &mut PandocEncodeContext,
) -> Vec<pandoc::Inline> {
    inlines
        .iter()
        .map(|inline| inline_to_pandoc(inline, context))
        .collect()
}

pub(super) fn inlines_from_pandoc(
    inlines: Vec<pandoc::Inline>,
    context: &mut PandocDecodeContext,
) -> Vec<Inline> {
    let mut inlines: Vec<Inline> = inlines
        .into_iter()
        .map(|inline| inline_from_pandoc(inline, context))
        .collect();

    // Pandoc splits strings up by whitespace. This combines adjacent strings.
    let mut index = 1;
    while index < inlines.len() {
        let curr = inlines[index].clone();
        match (&mut inlines[index - 1], curr) {
            (Inline::Text(Text { value: prev, .. }), Inline::Text(Text { value: curr, .. })) => {
                match curr.as_str() {
                    // TODO: check encoding below of line and soft breaks
                    "\u{2029}" => prev.push(' '),
                    _ => prev.push_str(&curr),
                };
                inlines.remove(index);
            }
            _ => {
                index += 1;
            }
        }
    }

    inlines
}

#[rustfmt::skip]
fn inline_to_pandoc(
    inline: &Inline,
    context: &mut PandocEncodeContext,
) -> pandoc::Inline {
    match inline {
        // Primitives
        Inline::Null(value) => pandoc::Inline::Str(value.to_string()),
        Inline::Boolean(value) => pandoc::Inline::Str(value.to_string()),
        Inline::Integer(value) => pandoc::Inline::Str(value.to_string()),
        Inline::UnsignedInteger(value) => pandoc::Inline::Str(value.to_string()),
        Inline::Number(value) => pandoc::Inline::Str(value.to_string()),

        // Types with a value
        Inline::Text(text) => pandoc::Inline::Str(text.value.to_string()),
        Inline::Date(date) => pandoc::Inline::Str(date.value.to_string()),
        Inline::Time(time) => pandoc::Inline::Str(time.value.to_string()),
        Inline::DateTime(datetime) => pandoc::Inline::Str(datetime.value.to_string()),
        Inline::Timestamp(timestamp) => pandoc::Inline::Str(timestamp.value.to_string()),
        Inline::Duration(duration) => pandoc::Inline::Str(duration.value.to_string()),

        // Marks
        Inline::Emphasis(mark) => pandoc::Inline::Emph(inlines_to_pandoc(&mark.content, context)),
        Inline::QuoteInline(mark) => pandoc::Inline::Quoted(pandoc::QuoteType::DoubleQuote, inlines_to_pandoc(&mark.content, context)),
        Inline::Strikeout(mark) => pandoc::Inline::Strikeout(inlines_to_pandoc(&mark.content, context)),
        Inline::Strong(mark) => pandoc::Inline::Strong(inlines_to_pandoc(&mark.content, context)),
        Inline::Subscript(mark) => pandoc::Inline::Subscript(inlines_to_pandoc(&mark.content, context)),
        Inline::Superscript(mark) => pandoc::Inline::Superscript(inlines_to_pandoc(&mark.content, context)),
        Inline::Underline(mark) => pandoc::Inline::Underline(inlines_to_pandoc(&mark.content, context)),

        // Media and links
        Inline::AudioObject(audio) => audio_to_pandoc(audio),
        Inline::ImageObject(image) => image_to_pandoc(image),
        Inline::MediaObject(media) => media_to_pandoc(media),
        Inline::VideoObject(video) => video_to_pandoc(video),
        Inline::Link(link) => link_to_pandoc(link),
        Inline::Cite(cite) => cite_to_pandoc(cite),
        Inline::CiteGroup(cite_group) => cite_group_to_pandoc(cite_group),

        // Code and math
        Inline::CodeExpression(code) => code_expression_to_pandoc(code, context),
        Inline::CodeInline(code) => code_inline_to_pandoc(code, context),
        Inline::MathInline(code) => math_inline_to_pandoc(code, context),

        // Other
        Inline::Note(note) => note_to_pandoc(note),
        Inline::StyledInline(styled) => styled_inline_to_pandoc(styled),

        // Inline types currently ignored: record loss and encode an empty span
        // TODO: implement these or remove from schema's `Inline` enum
        Inline::Button(..)
        | Inline::DeleteInline(..)
        | Inline::InsertInline(..)
        | Inline::InstructionInline(..)
        | Inline::ModifyInline(..)
        | Inline::Parameter(..)
        | Inline::ReplaceInline(..)
        | Inline::SuggestionInline(..) => {
            context.losses.add(inline.node_type().to_string());
            pandoc::Inline::Span(attrs_empty(), Vec::new())
        }
    }
}

#[rustfmt::skip]
fn inline_from_pandoc(inline: pandoc::Inline, context: &mut PandocDecodeContext) -> Inline {
    match inline {
        // Strings
        pandoc::Inline::Str(value) => Inline::Text(Text::new(value.into())),
        pandoc::Inline::Space => Inline::Text(Text::new(" ".into())),
        // TODO: encode as \n or UTF8?
        pandoc::Inline::LineBreak => Inline::Text(Text::new(" ".into())),
        pandoc::Inline::SoftBreak => Inline::Text(Text::new(" ".into())),

        // Marks
        pandoc::Inline::Emph(inlines) => Inline::Emphasis(Emphasis::new(inlines_from_pandoc(inlines, context))),
        pandoc::Inline::Quoted(_type, inlines) => Inline::QuoteInline(QuoteInline::new(inlines_from_pandoc(inlines, context))),
        pandoc::Inline::Strikeout(inlines) => Inline::Strikeout(Strikeout::new(inlines_from_pandoc(inlines, context))),
        pandoc::Inline::Strong(inlines) => Inline::Strong(Strong::new(inlines_from_pandoc(inlines, context))),
        pandoc::Inline::Subscript(inlines) => Inline::Subscript(Subscript::new(inlines_from_pandoc(inlines, context))),
        pandoc::Inline::Superscript(inlines) => Inline::Superscript(Superscript::new(inlines_from_pandoc(inlines, context))),
        pandoc::Inline::Underline(inlines) => Inline::Underline(Underline::new(inlines_from_pandoc(inlines, context))),
        // Note: Stencila does not have small caps yet, so use strong
        pandoc::Inline::SmallCaps(inlines) => Inline::Strong(Strong::new(inlines_from_pandoc(inlines, context))),

        // Media, links, citations
        pandoc::Inline::Image(attrs, inlines, target) => media_from_pandoc(attrs, inlines, target),
        pandoc::Inline::Link(attrs, inlines, target) => link_from_pandoc(attrs, inlines, target),
        pandoc::Inline::Cite(citations, inlines) => cite_from_pandoc(citations, inlines),
        
        // Code and math
        pandoc::Inline::Code(attrs, code) => code_inline_from_pandoc(attrs, code, context),
        pandoc::Inline::Math(_type, code) => math_inline_from_pandoc(code, context),

        // Other
        pandoc::Inline::Note(blocks) => note_from_pandoc(blocks),
        pandoc::Inline::Span(attrs, inlines ) => styled_inline_from_pandoc(attrs, inlines),
        // Note: Stencila does not have raw inline yet, so use code
        pandoc::Inline::RawInline(format, content) => Inline::CodeInline(CodeInline{ programming_language: Some(format.0), code: content.into(), ..Default::default()})
    }
}

fn audio_to_pandoc(audio: &AudioObject) -> pandoc::Inline {
    todo!()
}

fn image_to_pandoc(image: &ImageObject) -> pandoc::Inline {
    todo!()
}

fn video_to_pandoc(video: &VideoObject) -> pandoc::Inline {
    todo!()
}

fn media_to_pandoc(media: &MediaObject) -> pandoc::Inline {
    todo!()
}

fn media_from_pandoc(attrs: Attr, inlines: Vec<pandoc::Inline>, target: pandoc::Target) -> Inline {
    todo!()
}

fn link_to_pandoc(link: &Link) -> pandoc::Inline {
    todo!()
}

fn link_from_pandoc(attrs: Attr, inlines: Vec<pandoc::Inline>, target: pandoc::Target) -> Inline {
    todo!()
}

fn cite_to_pandoc(cite: &Cite) -> pandoc::Inline {
    todo!()
}

fn cite_group_to_pandoc(cite_group: &CiteGroup) -> pandoc::Inline {
    todo!()
}

fn cite_from_pandoc(citations: Vec<pandoc::Citation>, inlines: Vec<pandoc::Inline>) -> Inline {
    todo!()
}

fn note_to_pandoc(note: &Note) -> pandoc::Inline {
    todo!()
}

fn note_from_pandoc(blocks: Vec<pandoc::Block>) -> Inline {
    todo!()
}

fn styled_inline_to_pandoc(styled: &StyledInline) -> pandoc::Inline {
    todo!()
}

fn styled_inline_from_pandoc(attrs: Attr, inlines: Vec<pandoc::Inline>) -> Inline {
    todo!()
}

fn code_expression_to_pandoc(
    expr: &CodeExpression,
    _context: &mut PandocEncodeContext,
) -> pandoc::Inline {
    // Encode output value, if any, otherwise code
    let content = if let Some(output) = &expr.output {
        to_text(output)
    } else {
        expr.code.to_string()
    };
    pandoc::Inline::Code(attrs_empty(), content)
}

fn code_inline_to_pandoc(code: &CodeInline, _context: &mut PandocEncodeContext) -> pandoc::Inline {
    pandoc::Inline::Code(attrs_empty(), code.code.to_string())
}

fn code_inline_from_pandoc(
    attrs: Attr,
    code: String,
    _context: &mut PandocDecodeContext,
) -> Inline {
    Inline::CodeInline(CodeInline {
        code: code.into(),
        ..Default::default()
    })
}

fn math_inline_to_pandoc(math: &MathInline, _context: &mut PandocEncodeContext) -> pandoc::Inline {
    pandoc::Inline::Math(pandoc::MathType::InlineMath, math.code.to_string())
}

fn math_inline_from_pandoc(code: String, _context: &mut PandocDecodeContext) -> Inline {
    Inline::MathInline(MathInline {
        code: code.into(),
        ..Default::default()
    })
}
