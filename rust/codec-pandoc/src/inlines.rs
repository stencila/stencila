use pandoc_types::definition::{self as pandoc, Attr, Target};

use codec::{
    common::{once_cell::sync::Lazy, regex::Regex, serde_json},
    schema::*,
};
use codec_text_trait::to_text;

use crate::{
    blocks::{blocks_from_pandoc, blocks_to_pandoc},
    shared::{attrs_classes, attrs_empty, PandocDecodeContext, PandocEncodeContext},
};

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
        Inline::AudioObject(audio) => audio_to_pandoc(audio, context),
        Inline::ImageObject(image) => image_to_pandoc(image, context),
        Inline::MediaObject(media) => media_to_pandoc(media, context),
        Inline::VideoObject(video) => video_to_pandoc(video, context),
        Inline::Link(link) => link_to_pandoc(link, context),
        Inline::Cite(cite) => cite_to_pandoc(cite, context),
        Inline::CiteGroup(cite_group) => cite_group_to_pandoc(cite_group, context),

        // Code and math
        Inline::CodeExpression(code) => code_expression_to_pandoc(code, context),
        Inline::CodeInline(code) => code_inline_to_pandoc(code, context),
        Inline::MathInline(code) => math_inline_to_pandoc(code, context),

        // Other
        Inline::Note(note) => note_to_pandoc(note, context),
        Inline::StyledInline(styled) => styled_inline_to_pandoc(styled, context),
        Inline::Parameter(parameter) => parameter_to_pandoc(parameter, context),

        // Inline types currently ignored: record loss and encode an empty span
        // TODO: implement these or remove from schema's `Inline` enum
        _ => {
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
        pandoc::Inline::Image(attrs, inlines, target) => media_from_pandoc(attrs, inlines, target, context),
        pandoc::Inline::Link(attrs, inlines, target) => link_from_pandoc(attrs, inlines, target, context),
        pandoc::Inline::Cite(citations, inlines) => cite_from_pandoc(citations, inlines, context),
        
        // Code and math
        pandoc::Inline::Code(attrs, code) => code_inline_from_pandoc(attrs, code, context),
        pandoc::Inline::Math(_type, code) => math_inline_from_pandoc(code, context),

        // Other
        pandoc::Inline::Note(blocks) => note_from_pandoc(blocks, context),
        pandoc::Inline::Span(attrs, inlines ) => styled_inline_from_pandoc(attrs, inlines, context),
        // Note: Stencila does not have raw inline yet, so use code
        pandoc::Inline::RawInline(format, content) => inline_from_pandoc_raw_inline(format, content, context),
    }
}

fn audio_to_pandoc(audio: &AudioObject, context: &mut PandocEncodeContext) -> pandoc::Inline {
    media_object_to_pandoc(&audio.content_url, &audio.title, &audio.caption, context)
}

fn image_to_pandoc(image: &ImageObject, context: &mut PandocEncodeContext) -> pandoc::Inline {
    media_object_to_pandoc(&image.content_url, &image.title, &image.caption, context)
}

fn video_to_pandoc(video: &VideoObject, context: &mut PandocEncodeContext) -> pandoc::Inline {
    media_object_to_pandoc(&video.content_url, &video.title, &video.caption, context)
}

fn media_to_pandoc(media: &MediaObject, context: &mut PandocEncodeContext) -> pandoc::Inline {
    media_object_to_pandoc(&media.content_url, &media.options.title, &None, context)
}

fn media_object_to_pandoc(
    url: &str,
    title: &Option<Vec<Inline>>,
    caption: &Option<Vec<Inline>>,
    context: &mut PandocEncodeContext,
) -> pandoc::Inline {
    pandoc::Inline::Image(
        attrs_empty(),
        caption
            .as_ref()
            .map(|caption| inlines_to_pandoc(caption, context))
            .unwrap_or_default(),
        pandoc::Target {
            url: url.to_string(),
            title: title.as_ref().map(to_text).unwrap_or_default(),
        },
    )
}

fn media_from_pandoc(
    _attrs: Attr,
    inlines: Vec<pandoc::Inline>,
    target: pandoc::Target,
    context: &mut PandocDecodeContext,
) -> Inline {
    let content_url = target.url.to_string();
    let title =
        (!target.title.is_empty()).then(|| vec![Inline::Text(Text::new(target.title.into()))]);
    let caption = (!inlines.is_empty()).then(|| inlines_from_pandoc(inlines, context));

    Inline::ImageObject(ImageObject {
        content_url,
        title,
        caption,
        ..Default::default()
    })
}

fn link_to_pandoc(link: &Link, context: &mut PandocEncodeContext) -> pandoc::Inline {
    pandoc::Inline::Link(
        attrs_empty(),
        inlines_to_pandoc(&link.content, context),
        Target {
            url: link.target.clone(),
            title: link.title.clone().unwrap_or_default(),
        },
    )
}

fn link_from_pandoc(
    _attrs: Attr,
    inlines: Vec<pandoc::Inline>,
    target: pandoc::Target,
    context: &mut PandocDecodeContext,
) -> Inline {
    let title = (!target.title.is_empty()).then(|| target.title.clone());
    let target = target.url.to_string();
    let content = inlines_from_pandoc(inlines, context);

    Inline::Link(Link {
        target,
        title,
        content,
        ..Default::default()
    })
}

fn cite_to_pandoc(cite: &Cite, _context: &mut PandocEncodeContext) -> pandoc::Inline {
    pandoc::Inline::Cite(vec![cite_to_pandoc_citation(cite)], Vec::new())
}

fn cite_group_to_pandoc(
    cite_group: &CiteGroup,
    _context: &mut PandocEncodeContext,
) -> pandoc::Inline {
    pandoc::Inline::Cite(
        cite_group
            .items
            .iter()
            .map(cite_to_pandoc_citation)
            .collect(),
        Vec::new(),
    )
}

fn cite_to_pandoc_citation(cite: &Cite) -> pandoc::Citation {
    pandoc::Citation {
        citation_id: cite.target.clone(),
        citation_mode: match cite.citation_mode {
            CitationMode::Parenthetical => pandoc::CitationMode::NormalCitation,
            CitationMode::Narrative => pandoc::CitationMode::SuppressAuthor,
            CitationMode::NarrativeAuthor => pandoc::CitationMode::AuthorInText,
        },
        citation_prefix: cite
            .options
            .citation_prefix
            .as_ref()
            .map(|prefix| vec![pandoc::Inline::Str(prefix.into())])
            .unwrap_or_default(),
        citation_suffix: cite
            .options
            .citation_suffix
            .as_ref()
            .map(|suffix| vec![pandoc::Inline::Str(suffix.into())])
            .unwrap_or_default(),
        citation_note_num: 0,
        citation_hash: 0,
    }
}

fn cite_from_pandoc(
    citations: Vec<pandoc::Citation>,
    _inlines: Vec<pandoc::Inline>,
    context: &mut PandocDecodeContext,
) -> Inline {
    let mut cites: Vec<Cite> = citations
        .into_iter()
        .map(|cite| Cite {
            target: cite.citation_id,
            citation_mode: match cite.citation_mode {
                pandoc::CitationMode::NormalCitation => CitationMode::Parenthetical,
                pandoc::CitationMode::SuppressAuthor => CitationMode::Narrative,
                pandoc::CitationMode::AuthorInText => CitationMode::NarrativeAuthor,
            },
            options: Box::new(CiteOptions {
                citation_prefix: (!cite.citation_prefix.is_empty())
                    .then(|| to_text(&inlines_from_pandoc(cite.citation_prefix, context))),
                citation_suffix: (!cite.citation_suffix.is_empty())
                    .then(|| to_text(&inlines_from_pandoc(cite.citation_suffix, context))),
                ..Default::default()
            }),
            ..Default::default()
        })
        .collect();

    if cites.len() == 1 {
        Inline::Cite(cites.swap_remove(0))
    } else {
        Inline::CiteGroup(CiteGroup::new(cites))
    }
}

fn note_to_pandoc(note: &Note, context: &mut PandocEncodeContext) -> pandoc::Inline {
    pandoc::Inline::Note(blocks_to_pandoc(&note.content, context))
}

fn note_from_pandoc(blocks: Vec<pandoc::Block>, context: &mut PandocDecodeContext) -> Inline {
    Inline::Note(Note::new(
        NoteType::Footnote,
        blocks_from_pandoc(blocks, context),
    ))
}

fn styled_inline_to_pandoc(
    styled: &StyledInline,
    context: &mut PandocEncodeContext,
) -> pandoc::Inline {
    if styled.style_language.is_some() {
        context.losses.add("StyledBlock.styleLanguage");
    }

    let classes = styled.code.split(' ').map(String::from).collect();

    pandoc::Inline::Span(
        attrs_classes(classes),
        inlines_to_pandoc(&styled.content, context),
    )
}

fn styled_inline_from_pandoc(
    attrs: Attr,
    inlines: Vec<pandoc::Inline>,
    context: &mut PandocDecodeContext,
) -> Inline {
    Inline::StyledInline(StyledInline::new(
        attrs.classes.join(" ").into(),
        inlines_from_pandoc(inlines, context),
    ))
}

fn code_expression_to_pandoc(
    expr: &CodeExpression,
    _context: &mut PandocEncodeContext,
) -> pandoc::Inline {
    // Encode output value, if any, otherwise code with language in curly braces as prefix
    let content = if let Some(output) = &expr.output {
        to_text(output)
    } else {
        let lang = expr.programming_language.clone().unwrap_or_default();
        ["{", &lang, "} ", &expr.code].concat()
    };
    pandoc::Inline::Code(attrs_empty(), content)
}

fn code_inline_to_pandoc(code: &CodeInline, _context: &mut PandocEncodeContext) -> pandoc::Inline {
    pandoc::Inline::Code(attrs_empty(), code.code.to_string())
}

fn code_inline_from_pandoc(
    _attrs: Attr,
    code: String,
    _context: &mut PandocDecodeContext,
) -> Inline {
    // If the code starts with {lang} then treat as a code expression
    static REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^\{(\w+)?\}\s*(.*)").expect("invalid regex"));

    if let Some(captures) = REGEX.captures(&code) {
        let programming_language = captures.get(1).map(|lang| lang.as_str().to_string());
        Inline::CodeExpression(CodeExpression {
            programming_language,
            code: captures[2].into(),
            ..Default::default()
        })
    } else {
        Inline::CodeInline(CodeInline {
            code: code.into(),
            ..Default::default()
        })
    }
}

fn math_inline_to_pandoc(math: &MathInline, _context: &mut PandocEncodeContext) -> pandoc::Inline {
    pandoc::Inline::Math(pandoc::MathType::InlineMath, math.code.to_string())
}

fn math_inline_from_pandoc(code: String, _context: &mut PandocDecodeContext) -> Inline {
    Inline::MathInline(MathInline {
        code: code.into(),
        math_language: Some("tex".to_string()),
        ..Default::default()
    })
}

fn parameter_to_pandoc(param: &Parameter, _context: &mut PandocEncodeContext) -> pandoc::Inline {
    pandoc::Inline::RawInline(
        pandoc::Format("json".into()),
        serde_json::to_string(param).unwrap_or_default(),
    )
}

fn inline_from_pandoc_raw_inline(
    format: pandoc::Format,
    content: String,
    _context: &mut PandocDecodeContext,
) -> Inline {
    // This is currently used as a fallback for inline nodes that are not natively supported in Pandoc,
    // and not (yet) represented some other way.
    if format.0 == "json" {
        if let Ok(inline) = serde_json::from_str(&content) {
            return inline;
        }
    }

    Inline::CodeInline(CodeInline {
        programming_language: Some(format.0),
        code: content.into(),
        ..Default::default()
    })
}
