use codec::{
    format::Format,
    schema::{
        Annotation, CodeInline, Emphasis, Inline, Link, Strikeout, Strong, Subscript, Superscript,
        Text, Underline,
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

        lexical::InlineNode::Link(link) => link_from_lexical(link),
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
        Text(inline) => text_to_lexical(inline, context),
        Link(link) => link_to_lexical(link),

        _ => {
            context.losses.add(inline.node_type().to_string());
            lexical::InlineNode::Text(lexical::TextNode::default())
        }
    }
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

fn link_from_lexical(link: lexical::LinkNode) -> Inline {
    if link.target.is_some() {
        Inline::Link(Link {
            rel: link.rel,
            target: link.target.unwrap_or_default(),
            content: vec![Inline::Text(Text::from(link.url))],
            ..Default::default()
        })
    } else {
        Inline::Link(Link {
            rel: link.rel,
            target: link.url.clone(),
            content: vec![Inline::Text(Text::from(link.url))],
            ..Default::default()
        })
    }
}

fn link_to_lexical(link: &Link) -> lexical::InlineNode {
    if let Inline::Text(Text { value, .. }) = link.content.first().unwrap() {
        lexical::InlineNode::Link(lexical::LinkNode {
            url: value.to_string(),
            target: Some(link.target.clone()),
            rel: link.rel.clone(),
            ..Default::default()
        })
    } else {
        lexical::InlineNode::Link(lexical::LinkNode {
            target: Some(link.target.clone()),
            rel: link.rel.clone(),
            ..Default::default()
        })
    }
}
