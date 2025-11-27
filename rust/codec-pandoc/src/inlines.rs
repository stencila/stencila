use itertools::Itertools;
use pandoc_types::definition::{self as pandoc, Attr, Target};

use stencila_codec::{stencila_format::Format, stencila_schema::*};
use stencila_codec_text_trait::to_text;

use crate::{
    blocks::{blocks_from_pandoc, blocks_to_pandoc},
    shared::{PandocDecodeContext, PandocEncodeContext, attrs_classes, attrs_empty},
};

pub(super) fn inlines_to_pandoc(
    property: NodeProperty,
    inlines: &[Inline],
    context: &mut PandocEncodeContext,
) -> Vec<pandoc::Inline> {
    context.within_property(property, |context| {
        inlines
            .iter()
            .enumerate()
            .map(|(index, inline)| {
                context.within_index(index, |context| inline_to_pandoc(inline, context))
            })
            .collect()
    })
}

pub(super) fn inlines_from_pandoc(
    inlines: Vec<pandoc::Inline>,
    context: &mut PandocDecodeContext,
) -> Vec<Inline> {
    let mut inlines: Vec<Inline> = inlines
        .into_iter()
        .filter_map(|inline| inline_from_pandoc(inline, context))
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
pub(super) fn inline_to_pandoc(
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
        Inline::Emphasis(mark) => pandoc::Inline::Emph(inlines_to_pandoc(NodeProperty::Content, &mark.content, context)),
        Inline::QuoteInline(mark) => pandoc::Inline::Quoted(pandoc::QuoteType::DoubleQuote, inlines_to_pandoc(NodeProperty::Content, &mark.content, context)),
        Inline::Strikeout(mark) => pandoc::Inline::Strikeout(inlines_to_pandoc(NodeProperty::Content, &mark.content, context)),
        Inline::Strong(mark) => pandoc::Inline::Strong(inlines_to_pandoc(NodeProperty::Content, &mark.content, context)),
        Inline::Subscript(mark) => pandoc::Inline::Subscript(inlines_to_pandoc(NodeProperty::Content, &mark.content, context)),
        Inline::Superscript(mark) => pandoc::Inline::Superscript(inlines_to_pandoc(NodeProperty::Content, &mark.content, context)),
        Inline::Underline(mark) => pandoc::Inline::Underline(inlines_to_pandoc(NodeProperty::Content, &mark.content, context)),

        // Media and links
        Inline::AudioObject(audio) => audio_to_pandoc(audio, context),
        Inline::ImageObject(image) => image_to_pandoc(image, context),
        Inline::MediaObject(media) => media_to_pandoc(media, context),
        Inline::VideoObject(video) => video_to_pandoc(video, context),
        Inline::Link(link) => link_to_pandoc(link, context),
        Inline::Citation(citation) => citation_to_pandoc(citation, context),
        Inline::CitationGroup(citation_group) => citation_group_to_pandoc(citation_group, context),

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
fn inline_from_pandoc(inline: pandoc::Inline, context: &mut PandocDecodeContext) -> Option<Inline> {
    Some(match inline {
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
        pandoc::Inline::Cite(citations, inlines) => citation_from_pandoc(citations, inlines, context),
        
        // Code and math
        pandoc::Inline::Code(attrs, code) => code_inline_from_pandoc(attrs, code, context),
        pandoc::Inline::Math(_type, code) => math_inline_from_pandoc(code, context),

        // Other
        pandoc::Inline::Note(blocks) => note_from_pandoc(blocks, context),
        pandoc::Inline::Span(attrs, inlines ) => {
            if inlines.is_empty() {
                return None
            }
            styled_inline_from_pandoc(attrs, inlines, context)
        },
        // Note: Stencila does not have raw inline yet, so use code
        pandoc::Inline::RawInline(format, content) => inline_from_pandoc_raw_inline(format, content, context),
    })
}

fn audio_to_pandoc(audio: &AudioObject, context: &mut PandocEncodeContext) -> pandoc::Inline {
    media_object_to_pandoc(&audio.content_url, &audio.title, &audio.caption, context)
}

pub(super) fn image_to_pandoc(
    image: &ImageObject,
    context: &mut PandocEncodeContext,
) -> pandoc::Inline {
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
            .map(|caption| inlines_to_pandoc(NodeProperty::Caption, caption, context))
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

    let format = Format::from_url(&target.url);
    if format.is_audio() {
        Inline::AudioObject(AudioObject {
            content_url,
            caption,
            title,
            ..Default::default()
        })
    } else if format.is_video() {
        Inline::VideoObject(VideoObject {
            content_url,
            caption,
            title,
            ..Default::default()
        })
    } else {
        Inline::ImageObject(ImageObject {
            content_url,
            caption,
            title,
            ..Default::default()
        })
    }
}

fn link_to_pandoc(link: &Link, context: &mut PandocEncodeContext) -> pandoc::Inline {
    pandoc::Inline::Link(
        attrs_empty(),
        inlines_to_pandoc(NodeProperty::Content, &link.content, context),
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
    let mut target = target.url.to_string();
    let content = inlines_from_pandoc(inlines, context);

    // Some software will URL encode the target (e.g. LibreOffice for DOCX). This can break
    // internal links using ids like `fig:xxx` and `tab:xxx` as used in LaTeX documents so
    // revert these. Some software also places a %20 at the start of the URL.
    target = target.trim_start_matches("%20").to_string();
    if let Some(rest) = target.strip_prefix("#tab%3A") {
        target = format!("#tab:{rest}");
    } else if let Some(rest) = target.strip_prefix("#fig%3A") {
        target = format!("#fig:{rest}");
    } else if let Some(rest) = target.strip_prefix("#equ%3A") {
        target = format!("#equ:{rest}");
    } else if let Some(rest) = target.strip_prefix("#app%3A") {
        target = format!("#app:{rest}");
    }

    let label_only = if !target.starts_with("https://") && !target.starts_with("http://") {
        // Set to true if an internal link with only a number as content
        to_text(&content)
            .trim()
            .parse::<u32>()
            .is_ok()
            .then_some(true)
    } else {
        None
    };

    Inline::Link(Link {
        target,
        title,
        content,
        label_only,
        ..Default::default()
    })
}

fn citation_to_pandoc(cite: &Citation, _context: &mut PandocEncodeContext) -> pandoc::Inline {
    pandoc::Inline::Cite(vec![cite_to_pandoc_citation(cite)], Vec::new())
}

fn citation_group_to_pandoc(
    cite_group: &CitationGroup,
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

fn cite_to_pandoc_citation(cite: &Citation) -> pandoc::Citation {
    pandoc::Citation {
        citation_id: cite.target.clone(),
        citation_mode: match cite.citation_mode {
            Some(CitationMode::Narrative) => pandoc::CitationMode::SuppressAuthor,
            Some(CitationMode::NarrativeAuthor) => pandoc::CitationMode::AuthorInText,
            _ => pandoc::CitationMode::NormalCitation,
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

fn citation_from_pandoc(
    citations: Vec<pandoc::Citation>,
    _inlines: Vec<pandoc::Inline>,
    context: &mut PandocDecodeContext,
) -> Inline {
    let single = citations.len() == 1;

    let mut cites: Vec<Citation> = citations
        .into_iter()
        .map(|cite| Citation {
            target: cite.citation_id,
            citation_mode: match cite.citation_mode {
                pandoc::CitationMode::NormalCitation => {
                    if single {
                        Some(CitationMode::Parenthetical)
                    } else {
                        None
                    }
                }
                pandoc::CitationMode::SuppressAuthor => Some(CitationMode::Narrative),
                pandoc::CitationMode::AuthorInText => Some(CitationMode::NarrativeAuthor),
            },
            options: Box::new(CitationOptions {
                citation_prefix: (!cite.citation_prefix.is_empty())
                    .then(|| to_text(&inlines_from_pandoc(cite.citation_prefix, context))),
                citation_suffix: (!cite.citation_suffix.is_empty())
                    .then(|| to_text(&inlines_from_pandoc(cite.citation_suffix, context))),
                ..Default::default()
            }),
            ..Default::default()
        })
        .collect();

    if single {
        Inline::Citation(cites.swap_remove(0))
    } else {
        Inline::CitationGroup(CitationGroup::new(cites))
    }
}

fn note_to_pandoc(note: &Note, context: &mut PandocEncodeContext) -> pandoc::Inline {
    pandoc::Inline::Note(blocks_to_pandoc(
        NodeProperty::Content,
        &note.content,
        context,
    ))
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
        inlines_to_pandoc(NodeProperty::Content, &styled.content, context),
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
    context: &mut PandocEncodeContext,
) -> pandoc::Inline {
    if context.render {
        let Some(output) = &expr.output else {
            return if context.reproducible {
                // If no output and reproducible then just a repro-link
                context.reproducible_link(
                    NodeType::CodeExpression,
                    expr,
                    None,
                    pandoc::Inline::Str("[Code Expression]".into()),
                )
            } else {
                // If no output and not reproducible then nothing
                pandoc::Inline::Str("".into())
            };
        };

        let inline = pandoc::Inline::Str(to_text(output));

        return if context.reproducible {
            context.reproducible_link(NodeType::CodeExpression, expr, None, inline)
        } else {
            context.output_span(inline)
        };
    }

    // If not render mode, and if these formats, then encode empty span
    if matches!(context.format, Format::Docx | Format::Odt) {
        return pandoc::Inline::Span(attrs_empty(), Vec::new());
    }

    // If LaTeX, encode as special command
    if matches!(context.format, Format::Latex | Format::Rnw) {
        let begin = if matches!(context.format, Format::Rnw) {
            "\\Sexpr{"
        } else {
            "\\expr{"
        };
        return pandoc::Inline::RawInline(
            pandoc::Format("latex".into()),
            [begin, &expr.code.to_string(), "}"].concat(),
        );
    }

    // Otherwise, encode as static inline code
    let mut lang = expr.programming_language.clone().unwrap_or_default();
    lang.push_str("exec");

    pandoc::Inline::Code(attrs_classes(vec![lang]), expr.code.to_string())
}

fn code_inline_to_pandoc(code: &CodeInline, _context: &mut PandocEncodeContext) -> pandoc::Inline {
    pandoc::Inline::Code(attrs_empty(), code.code.to_string())
}

fn code_inline_from_pandoc(
    attrs: Attr,
    code: String,
    _context: &mut PandocDecodeContext,
) -> Inline {
    // Note: Pandoc currently only observes the `language` option
    // (and puts its value into the `attrs.classes`). This differs to the handling
    // of code blocks (in which all attributes are preserved in `attrs.attributes`).
    // For that reason we rely on the language name being "exec", or suffixed with "exec",
    // to be able to identify code expressions
    let programming_language = attrs.classes.first().cloned();
    if let Some(lang) = programming_language
        .as_ref()
        .and_then(|lang| lang.strip_suffix("exec"))
    {
        let lang = (!lang.is_empty()).then_some(lang.into());

        return Inline::CodeExpression(CodeExpression {
            programming_language: lang,
            code: code.into(),
            ..Default::default()
        });
    }

    Inline::CodeInline(CodeInline {
        programming_language,
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
    if format.0 == "json"
        && let Ok(inline) = serde_json::from_str(&content)
    {
        return inline;
    }

    Inline::CodeInline(CodeInline {
        programming_language: Some(format.0),
        code: content.into(),
        ..Default::default()
    })
}

pub(super) fn string_from_pandoc_inlines(inlines: Vec<pandoc::Inline>) -> String {
    inlines.into_iter().map(string_from_pandoc_inline).join("")
}

fn string_from_pandoc_inline(inline: pandoc::Inline) -> String {
    match inline {
        pandoc::Inline::Str(value) => value,
        pandoc::Inline::Space => " ".into(),
        pandoc::Inline::LineBreak => " ".into(),
        pandoc::Inline::SoftBreak => " ".into(),

        pandoc::Inline::Emph(inlines)
        | pandoc::Inline::Quoted(.., inlines)
        | pandoc::Inline::Strikeout(inlines)
        | pandoc::Inline::Strong(inlines)
        | pandoc::Inline::Subscript(inlines)
        | pandoc::Inline::Superscript(inlines)
        | pandoc::Inline::Underline(inlines)
        | pandoc::Inline::SmallCaps(inlines)
        | pandoc::Inline::Image(.., inlines, _)
        | pandoc::Inline::Link(.., inlines, _)
        | pandoc::Inline::Cite(.., inlines)
        | pandoc::Inline::Span(.., inlines) => string_from_pandoc_inlines(inlines),

        pandoc::Inline::Code(.., code) | pandoc::Inline::Math(.., code) => code,

        pandoc::Inline::RawInline(.., content) => content,

        _ => "".into(),
    }
}
