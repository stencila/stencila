use codec::{
    common::tracing,
    format::Format,
    schema::{
        Annotation, CodeInline, Emphasis, Inline, Link, MathInline, Strikeout, Strong, Subscript,
        Superscript, Text, Underline,
    },
};

use crate::{
    lexical::{self, TextFormat},
    shared::{LexicalDecodeContext, LexicalEncodeContext},
};

pub(super) fn inlines_from_lexical(
    inlines: Vec<lexical::InlineNode>,
    context: &mut LexicalDecodeContext,
) -> Vec<Inline> {
    inlines
        .into_iter()
        .map(|inline| inline_from_lexical(inline, context))
        .collect()
}

pub(super) fn inlines_to_lexical(
    inlines: &[Inline],
    context: &mut LexicalEncodeContext,
) -> Vec<lexical::InlineNode> {
    inlines
        .iter()
        .flat_map(|inline| match &inline {
            Inline::Strong(Strong { content, .. }) => {
                formatted_to_lexical(TextFormat::BOLD, content, context)
            }
            Inline::Emphasis(Emphasis { content, .. }) => {
                formatted_to_lexical(TextFormat::ITALIC, content, context)
            }
            Inline::Strikeout(Strikeout { content, .. }) => {
                formatted_to_lexical(TextFormat::STRIKETHROUGH, content, context)
            }
            Inline::Underline(Underline { content, .. }) => {
                formatted_to_lexical(TextFormat::UNDERLINE, content, context)
            }
            Inline::Subscript(Subscript { content, .. }) => {
                formatted_to_lexical(TextFormat::SUBSCRIPT, content, context)
            }
            Inline::Superscript(Superscript { content, .. }) => {
                formatted_to_lexical(TextFormat::SUPERSCRIPT, content, context)
            }
            Inline::Annotation(Annotation { content, .. }) => {
                formatted_to_lexical(TextFormat::HIGHLIGHT, content, context)
            }
            Inline::CodeInline(CodeInline { code, .. }) => formatted_to_lexical(
                TextFormat::CODE,
                &vec![Inline::Text(Text::new(code.clone()))],
                context,
            ),
            _ => vec![inline_to_lexical(inline, context)],
        })
        .collect()
}

fn inline_from_lexical(inline: lexical::InlineNode, context: &mut LexicalDecodeContext) -> Inline {
    // Macro to indicate type that has not yet been implemented
    macro_rules! loss {
        ($name:expr) => {{
            context.losses.add($name);
            Inline::Text(Text::from(format!("LOST {}", $name)))
        }};
    }

    match inline {
        lexical::InlineNode::Text(lexical::TextNode { format, text, .. })
        | lexical::InlineNode::ExtendedText(lexical::ExtendedTextNode { format, text, .. }) => {
            text_from_lexical(format, text)
        }

        lexical::InlineNode::Link(link) => link_from_lexical(link, context),
        lexical::InlineNode::HashTag(..) => loss!("HashTag"),

        lexical::InlineNode::Unknown(inline) => {
            let typename = inline
                .get("type")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            loss!(format!("Unknown ({typename})"))
        }
    }
}

fn inline_to_lexical(inline: &Inline, context: &mut LexicalEncodeContext) -> lexical::InlineNode {
    use Inline::*;
    match inline {
        Link(link) => link_to_lexical(link, context),
        MathInline(math) => math_to_lexical(math),
        Text(inline) => text_to_lexical(inline, context),

        _ => {
            context.losses.add(inline.node_type().to_string());
            lexical::InlineNode::Text(lexical::TextNode::default())
        }
    }
}

fn link_from_lexical(link: lexical::LinkNode, context: &mut LexicalDecodeContext) -> Inline {
    Inline::Link(Link {
        title: link.title,
        rel: link.rel,
        target: link.url,
        content: inlines_from_lexical(link.children, context),
        ..Default::default()
    })
}

fn link_to_lexical(link: &Link, context: &mut LexicalEncodeContext) -> lexical::InlineNode {
    let children = inlines_to_lexical(&link.content, context);
    let url = link.target.clone();
    let title = link.title.clone();
    let rel = link.rel.clone();

    lexical::InlineNode::Link(lexical::LinkNode {
        children,
        url,
        title,
        rel,
        ..Default::default()
    })
}

fn math_to_lexical(math: &MathInline) -> lexical::InlineNode {
    // Lexical & Koenig do not support inline math, nor inline HTML, so encode as in Markdown
    // (using dollar delimiters). Then, at least the math will be editable.
    // With Ghost, if the theme includes KaTex, the math will also be rendered.

    if let Some(lang) = &math.math_language {
        let lang = lang.to_lowercase();
        if !(lang == "tex" || lang == "latex") {
            tracing::warn!("Math written in `{lang}` may not render correctly")
        }
    }

    lexical::InlineNode::Text(lexical::TextNode {
        text: ["$", &math.code, "$"].concat(),
        ..Default::default()
    })
}

fn text_from_lexical(format: lexical::TextFormat, value: String) -> Inline {
    if format.contains(TextFormat::CODE) {
        return Inline::CodeInline(CodeInline::new(value.into()));
    }

    let mut inline = Inline::Text(Text::from(value));

    if format.contains(TextFormat::BOLD) {
        inline = Inline::Strong(Strong::new(vec![inline]))
    }
    if format.contains(TextFormat::ITALIC) {
        inline = Inline::Emphasis(Emphasis::new(vec![inline]))
    }
    if format.contains(TextFormat::STRIKETHROUGH) {
        inline = Inline::Strikeout(Strikeout::new(vec![inline]))
    }
    if format.contains(TextFormat::UNDERLINE) {
        inline = Inline::Underline(Underline::new(vec![inline]))
    }
    if format.contains(TextFormat::SUBSCRIPT) {
        inline = Inline::Subscript(Subscript::new(vec![inline]))
    }
    if format.contains(TextFormat::SUPERSCRIPT) {
        inline = Inline::Superscript(Superscript::new(vec![inline]))
    }
    if format.contains(TextFormat::HIGHLIGHT) {
        inline = Inline::Annotation(Annotation::new(vec![inline]))
    }

    inline
}

fn formatted_to_lexical(
    format: lexical::TextFormat,
    inlines: &[Inline],
    context: &mut LexicalEncodeContext,
) -> Vec<lexical::InlineNode> {
    // Add the format to the context so it is applied to child inlines
    context.text_format.insert(format);

    let inlines = inlines_to_lexical(inlines, context);

    // Remove the format from the context
    context.text_format.remove(format);

    inlines
}

fn text_to_lexical(text: &Text, context: &mut LexicalEncodeContext) -> lexical::InlineNode {
    let format = context.text_format;
    let text = text.value.to_string();

    match context.format {
        Format::Koenig => lexical::InlineNode::ExtendedText(lexical::ExtendedTextNode {
            format,
            text,
            ..Default::default()
        }),
        _ => lexical::InlineNode::Text(lexical::TextNode {
            format,
            text,
            ..Default::default()
        }),
    }
}
