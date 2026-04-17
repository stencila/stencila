use itertools::Itertools;
use pandoc_types::definition::{self as pandoc, Attr, Target};

use stencila_codec::{stencila_format::Format, stencila_schema::*};
use stencila_codec_text_trait::to_text;

use crate::{
    blocks::{blocks_from_pandoc, blocks_to_pandoc},
    shared::{
        PandocDecodeContext, PandocEncodeContext, PendingComment, append_suggestion_attrs,
        attrs_classes, attrs_empty, decode_suggestion_attrs,
    },
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
            .flat_map(|(index, inline)| {
                context.within_index(index, |context| inline_to_pandoc_vec(inline, context))
            })
            .collect()
    })
}

fn inline_to_pandoc_vec(inline: &Inline, context: &mut PandocEncodeContext) -> Vec<pandoc::Inline> {
    match inline {
        Inline::SuggestionInline(suggestion) => {
            suggestion_inline_to_pandoc_inlines(suggestion, context)
        }
        _ => vec![inline_to_pandoc(inline, context)],
    }
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
        Inline::Boundary(boundary) => boundary_to_pandoc(boundary, context),
        Inline::SuggestionInline(suggestion) => suggestion_inline_to_pandoc(suggestion, context),

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
        pandoc::Inline::LineBreak => Inline::Text(Text::new("\n".into())),
        pandoc::Inline::SoftBreak => Inline::Text(Text::new("\n".into())),

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
            if attrs.classes.iter().any(|c| c == "comment-start") {
                return comment_start_from_pandoc(attrs, inlines, context);
            }
            if attrs.classes.iter().any(|c| c == "comment-end") {
                return comment_end_from_pandoc(attrs, inlines, context);
            }
            if inlines.is_empty() {
                return None
            }
            if attrs.classes.iter().any(|c| c == "insertion" || c == "deletion") {
                suggestion_inline_from_pandoc(attrs, inlines, context)
            } else {
                styled_inline_from_pandoc(attrs, inlines, context)
            }
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
    // For cloud doc formats (GDocx, M365Docx), convert local file paths to stencila.link URLs
    // to preserve them through round-tripping and enable GitHub permalinks
    let url = if matches!(context.format, Format::GDocx | Format::M365Docx)
        && is_file_target(&link.target)
    {
        context.file_url(&link.target)
    } else {
        link.target.clone()
    };

    pandoc::Inline::Link(
        attrs_empty(),
        inlines_to_pandoc(NodeProperty::Content, &link.content, context),
        Target {
            url,
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
    target = restore_internal_target_prefix(&target);

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

fn boundary_to_pandoc(boundary: &Boundary, context: &mut PandocEncodeContext) -> pandoc::Inline {
    let Some(id) = &boundary.id else {
        context.losses.add("Boundary.id");
        return pandoc::Inline::Span(attrs_empty(), Vec::new());
    };

    if let Some(span) = context.comment_start_spans.get(id) {
        return span.clone();
    }

    if let Some(span) = context.comment_end_spans.get(id) {
        return span.clone();
    }

    context.losses.add("Boundary");
    pandoc::Inline::Span(attrs_empty(), Vec::new())
}

pub(super) fn suggestion_inline_to_pandoc_inlines(
    suggestion: &SuggestionInline,
    context: &mut PandocEncodeContext,
) -> Vec<pandoc::Inline> {
    match suggestion.suggestion_type {
        Some(SuggestionType::Replace) => {
            let delete = pandoc_suggestion_span(
                vec!["deletion".into()],
                &suggestion.authors,
                &suggestion.date_published,
                suggestion.original.as_deref().unwrap_or_default(),
                NodeProperty::Original,
                context,
                "SuggestionInline.original",
            );
            let insert = pandoc_suggestion_span(
                vec!["insertion".into()],
                &suggestion.authors,
                &suggestion.date_published,
                &suggestion.content,
                NodeProperty::Content,
                context,
                "SuggestionInline.content",
            );
            vec![delete, insert]
        }
        Some(SuggestionType::Delete) => vec![pandoc_suggestion_span(
            vec!["deletion".into()],
            &suggestion.authors,
            &suggestion.date_published,
            &suggestion.content,
            NodeProperty::Content,
            context,
            "SuggestionInline.content",
        )],
        Some(SuggestionType::Insert) | None => vec![pandoc_suggestion_span(
            vec!["insertion".into()],
            &suggestion.authors,
            &suggestion.date_published,
            &suggestion.content,
            NodeProperty::Content,
            context,
            "SuggestionInline.content",
        )],
    }
}

fn pandoc_suggestion_span(
    classes: Vec<String>,
    authors: &Option<Vec<Author>>,
    date_published: &Option<DateTime>,
    content: &[Inline],
    property: NodeProperty,
    context: &mut PandocEncodeContext,
    empty_loss: &str,
) -> pandoc::Inline {
    let mut attrs = attrs_classes(classes);
    append_suggestion_attrs(&mut attrs, authors, date_published);

    if content.is_empty() {
        context.losses.add(empty_loss);
    }

    pandoc::Inline::Span(attrs, inlines_to_pandoc(property, content, context))
}

fn suggestion_inline_to_pandoc(
    suggestion: &SuggestionInline,
    context: &mut PandocEncodeContext,
) -> pandoc::Inline {
    if suggestion.suggestion_status.is_some() {
        context.losses.add("SuggestionInline.suggestionStatus");
    }
    if suggestion.provenance.is_some() {
        context.losses.add("SuggestionInline.provenance");
    }

    suggestion_inline_to_pandoc_inlines(suggestion, context)
        .into_iter()
        .next()
        .unwrap_or_else(|| pandoc::Inline::Span(attrs_empty(), Vec::new()))
}

fn suggestion_inline_from_pandoc(
    attrs: Attr,
    inlines: Vec<pandoc::Inline>,
    context: &mut PandocDecodeContext,
) -> Inline {
    let decoded_attrs = decode_suggestion_attrs(&attrs);

    Inline::SuggestionInline(SuggestionInline {
        suggestion_type: Some(decoded_attrs.suggestion_type),
        authors: decoded_attrs.authors,
        date_published: decoded_attrs.date_published,
        content: inlines_from_pandoc(inlines, context),
        ..Default::default()
    })
}

pub(super) fn merge_adjacent_replacement_suggestions(inlines: &mut Vec<Inline>) {
    let mut merged = Vec::with_capacity(inlines.len());
    let mut index = 0;

    while index < inlines.len() {
        let Some(Inline::SuggestionInline(delete)) = inlines.get(index) else {
            merged.push(inlines[index].clone());
            index += 1;
            continue;
        };

        let is_delete = delete.suggestion_type == Some(SuggestionType::Delete);
        let Some(Inline::SuggestionInline(insert)) = inlines.get(index + 1) else {
            merged.push(inlines[index].clone());
            index += 1;
            continue;
        };

        let is_insert = insert.suggestion_type == Some(SuggestionType::Insert);

        if is_delete
            && is_insert
            && delete.authors == insert.authors
            && delete.date_published == insert.date_published
            && delete.suggestion_status == insert.suggestion_status
            && delete.provenance == insert.provenance
            && delete.feedback == insert.feedback
        {
            merged.push(Inline::SuggestionInline(SuggestionInline {
                suggestion_type: Some(SuggestionType::Replace),
                date_published: delete.date_published.clone(),
                suggestion_status: delete.suggestion_status,
                authors: delete.authors.clone(),
                provenance: delete.provenance.clone(),
                feedback: delete.feedback.clone(),
                content: insert.content.clone(),
                original: Some(delete.content.clone()),
                ..Default::default()
            }));
            index += 2;
        } else {
            merged.push(inlines[index].clone());
            index += 1;
        }
    }

    *inlines = merged;
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

/// Get a value from the key-value attributes of a Pandoc [`Attr`].
///
/// Unlike [`get_attr`], this always searches `attrs.attributes` — even for
/// "id" — because Pandoc comment spans store the comment identifier as a
/// regular attribute, not in `attrs.identifier`.
fn get_kv_attr(attrs: &Attr, name: &str) -> Option<String> {
    attrs
        .attributes
        .iter()
        .find_map(|(k, v)| (k == name).then(|| v.clone()))
}

fn restore_internal_target_prefix(target: &str) -> String {
    for prefix in ["tab", "fig", "equ", "app"] {
        let encoded_prefix = format!("#{prefix}%3A");
        if let Some(rest) = target.strip_prefix(&encoded_prefix) {
            return format!("#{prefix}:{rest}");
        }
    }

    target.to_string()
}

pub(super) fn comment_blocks_to_pandoc_inlines(
    blocks: &[Block],
    context: &mut PandocEncodeContext,
) -> Vec<pandoc::Inline> {
    let mut inlines = Vec::new();

    for (index, block) in blocks.iter().enumerate() {
        match block {
            Block::Paragraph(paragraph) => {
                inlines.extend(inlines_to_pandoc(
                    NodeProperty::Content,
                    &paragraph.content,
                    context,
                ));
            }
            _ => {
                context.losses.add("Comment.content");
            }
        }

        if index + 1 < blocks.len() {
            inlines.push(pandoc::Inline::LineBreak);
        }
    }

    inlines
}

/// Handle a Pandoc `comment-start` span.
///
/// Emits a [`Boundary`] inline node and collects the comment metadata
/// and body inlines into the decode context for later assembly.
fn comment_start_from_pandoc(
    attrs: Attr,
    body_inlines: Vec<pandoc::Inline>,
    context: &mut PandocDecodeContext,
) -> Option<Inline> {
    let pandoc_id = comment_span_id(&attrs, context)?;
    let author = get_kv_attr(&attrs, "author");
    let date = get_kv_attr(&attrs, "date");

    let boundary_id = format!("comment-{pandoc_id}-start");

    context.pending_comments.push(PendingComment {
        pandoc_id,
        author,
        date,
        body_inlines,
        parent_pandoc_id: None,
    });

    Some(Inline::Boundary(Boundary {
        id: Some(boundary_id),
        ..Default::default()
    }))
}

fn comment_span_id(attrs: &Attr, context: &mut PandocDecodeContext) -> Option<String> {
    let Some(pandoc_id) = get_kv_attr(attrs, "id") else {
        context.losses.add("Comment.id");
        return None;
    };

    if pandoc_id.is_empty() {
        context.losses.add("Comment.id");
        return None;
    }

    Some(pandoc_id)
}

/// Handle a Pandoc `comment-end` span.
///
/// Emits [`Boundary`] inline nodes for this comment and any nested
/// comment-end spans (which indicate reply relationships).
fn comment_end_from_pandoc(
    attrs: Attr,
    inlines: Vec<pandoc::Inline>,
    context: &mut PandocDecodeContext,
) -> Option<Inline> {
    let pandoc_id = comment_span_id(&attrs, context)?;
    let boundary_id = format!("comment-{pandoc_id}-end");
    let parent_id = get_kv_attr(&attrs, "parent");

    if let Some(parent_id) = parent_id {
        if let Some(pending) = context
            .pending_comments
            .iter_mut()
            .find(|c| c.pandoc_id == pandoc_id)
        {
            pending.parent_pandoc_id = Some(parent_id);
            if pending.body_inlines.is_empty() {
                pending.body_inlines = inlines.clone();
            }
        } else {
            context.pending_comments.push(PendingComment {
                pandoc_id: pandoc_id.clone(),
                author: get_kv_attr(&attrs, "author"),
                date: get_kv_attr(&attrs, "date"),
                body_inlines: inlines.clone(),
                parent_pandoc_id: get_kv_attr(&attrs, "parent"),
            });
        }

        // Reply comments are metadata carriers nested inside another comment's
        // end span; they do not correspond to a boundary in article content.
        return None;
    }

    // Process any nested comment-end spans. Pandoc may deeply nest sibling
    // replies from DOCX. Those nested spans do not carry enough information to
    // distinguish true nested replies from flattened sibling replies, so leave
    // their parent unresolved here and normalize the relationships later.
    for inline in inlines {
        if let pandoc::Inline::Span(nested_attrs, nested_inlines) = inline
            && nested_attrs.classes.iter().any(|c| c == "comment-end")
        {
            let Some(nested_id) = comment_span_id(&nested_attrs, context) else {
                continue;
            };

            let nested_parent_id = get_kv_attr(&nested_attrs, "parent");

            // Mark the nested comment as a reply to this one, creating a pending
            // comment if needed. Unlike regular comments, reply comments may be
            // represented only by a nested `comment-end` span, with their body
            // carried in that span's inline content.
            if let Some(pending) = context
                .pending_comments
                .iter_mut()
                .find(|c| c.pandoc_id == nested_id)
            {
                if pending.parent_pandoc_id.is_none() {
                    pending.parent_pandoc_id = nested_parent_id.clone();
                }
                if pending.body_inlines.is_empty() {
                    pending.body_inlines = nested_inlines.clone();
                }
            } else {
                context.pending_comments.push(PendingComment {
                    pandoc_id: nested_id.clone(),
                    author: get_kv_attr(&nested_attrs, "author"),
                    date: get_kv_attr(&nested_attrs, "date"),
                    body_inlines: nested_inlines.clone(),
                    parent_pandoc_id: nested_parent_id,
                });
            }

            // Recursively handle deeper nesting
            comment_end_from_pandoc(nested_attrs, nested_inlines, context);
        }
    }

    Some(Inline::Boundary(Boundary {
        id: Some(boundary_id),
        ..Default::default()
    }))
}

/// Convert Pandoc inlines into blocks, splitting on [`LineBreak`] to create
/// separate paragraphs.
pub(super) fn pandoc_inlines_to_blocks(
    inlines: Vec<pandoc::Inline>,
    context: &mut PandocDecodeContext,
) -> Vec<Block> {
    // Split the inlines at each LineBreak into groups, one per paragraph
    let mut groups: Vec<Vec<pandoc::Inline>> = vec![Vec::new()];
    for inline in inlines {
        if matches!(inline, pandoc::Inline::LineBreak) {
            groups.push(Vec::new());
        } else if let Some(current) = groups.last_mut() {
            current.push(inline);
        }
    }

    groups
        .into_iter()
        .filter(|group| !group.is_empty())
        .map(|group| {
            Block::Paragraph(Paragraph {
                content: inlines_from_pandoc(group, context),
                ..Default::default()
            })
        })
        .collect()
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
