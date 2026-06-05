//! Inline-level conversion between Tiptap JSON nodes and Stencila inlines.
//!
//! The native inline mapping currently covers text and bold/italic marks only.
//! Unsupported Stencila inlines are preserved as custom opaque `stencilaInline`
//! nodes, while unsupported native Tiptap inlines and marks are recorded as
//! losses until explicit mappings are added.

use serde_json::Value;
use stencila_codec::stencila_schema::{Emphasis, Inline, Strong, Text};

use crate::{
    shared::{MarkKind, TiptapDecodeContext, TiptapEncodeContext},
    tiptap::{
        self, InlineNode, KnownMark, Mark, MarkType, StencilaAttrs, StencilaInlineNode, TextNode,
    },
};

/// Decode Tiptap inline nodes into Stencila inlines.
pub(super) fn inlines_from_tiptap(
    inlines: Vec<InlineNode>,
    context: &mut TiptapDecodeContext,
) -> Vec<Inline> {
    inlines
        .into_iter()
        .map(|inline| inline_from_tiptap(inline, context))
        .collect()
}

/// Encode Stencila inlines into Tiptap inline nodes.
pub(super) fn inlines_to_tiptap(
    inlines: &[Inline],
    context: &mut TiptapEncodeContext,
) -> Vec<InlineNode> {
    let nodes = inlines_to_tiptap_with_marks(inlines, &[], context);
    merge_adjacent_text_nodes(nodes)
}

fn inlines_to_tiptap_with_marks(
    inlines: &[Inline],
    marks: &[MarkKind],
    context: &mut TiptapEncodeContext,
) -> Vec<InlineNode> {
    inlines
        .iter()
        .flat_map(|inline| inline_to_tiptap(inline, marks, context))
        .collect()
}

fn inline_from_tiptap(inline: InlineNode, context: &mut TiptapDecodeContext) -> Inline {
    match inline {
        InlineNode::Text(TextNode { text, marks, .. }) => text_from_tiptap(text, marks, context),

        InlineNode::StencilaInline(StencilaInlineNode { attrs, .. }) => {
            inline_from_stencila_attrs(attrs, context)
        }

        InlineNode::Unknown(value) => {
            context
                .losses
                .add(format!("Unknown ({})", tiptap::value_type(&value)));
            Inline::Text(Text::default())
        }
    }
}

fn inline_to_tiptap(
    inline: &Inline,
    marks: &[MarkKind],
    context: &mut TiptapEncodeContext,
) -> Vec<InlineNode> {
    match inline {
        Inline::Text(text) => text_to_tiptap(text, marks),

        Inline::Strong(Strong { content, .. }) => {
            let marks = add_mark(marks, MarkKind::Bold);
            inlines_to_tiptap_with_marks(content, &marks, context)
        }

        Inline::Emphasis(Emphasis { content, .. }) => {
            let marks = add_mark(marks, MarkKind::Italic);
            inlines_to_tiptap_with_marks(content, &marks, context)
        }

        _ => vec![opaque_inline_to_tiptap(inline, context)],
    }
}

fn text_from_tiptap(value: String, marks: Vec<Mark>, context: &mut TiptapDecodeContext) -> Inline {
    let mut bold = false;
    let mut italic = false;

    for mark in marks {
        match mark {
            Mark::Known(KnownMark {
                r#type: MarkType::Bold,
            }) => bold = true,
            Mark::Known(KnownMark {
                r#type: MarkType::Italic,
            }) => italic = true,
            Mark::Unknown(value) => {
                context
                    .losses
                    .add(format!("Unknown mark ({})", tiptap::value_type(&value)));
            }
        }
    }

    let mut inline = Inline::Text(Text::new(value.into()));
    if bold {
        inline = Inline::Strong(Strong::new(vec![inline]));
    }
    if italic {
        inline = Inline::Emphasis(Emphasis::new(vec![inline]));
    }

    inline
}

fn text_to_tiptap(text: &Text, marks: &[MarkKind]) -> Vec<InlineNode> {
    let value = text.value.to_string();

    if value.is_empty() {
        Vec::new()
    } else {
        vec![InlineNode::Text(TextNode {
            text: value,
            marks: marks_to_tiptap(marks),
            r#type: Default::default(),
        })]
    }
}

fn opaque_inline_to_tiptap(inline: &Inline, context: &mut TiptapEncodeContext) -> InlineNode {
    match serde_json::to_value(inline) {
        Ok(node) => InlineNode::StencilaInline(StencilaInlineNode {
            attrs: StencilaAttrs {
                node_type: inline.node_type().to_string(),
                node,
            },
            r#type: Default::default(),
        }),
        Err(error) => {
            context
                .losses
                .add(format!("{}: {error}", inline.node_type()));
            InlineNode::Text(TextNode {
                text: String::new(),
                marks: Vec::new(),
                r#type: Default::default(),
            })
        }
    }
}

fn inline_from_stencila_attrs(attrs: StencilaAttrs, context: &mut TiptapDecodeContext) -> Inline {
    match serde_json::from_value::<Inline>(attrs.node) {
        Ok(inline) => {
            let node_type = inline.node_type().to_string();
            if node_type != attrs.node_type {
                context.losses.add(format!(
                    "StencilaInline.nodeType (expected {}, got {node_type})",
                    attrs.node_type
                ));
            }
            inline
        }
        Err(error) => {
            context.losses.add(format!("{}: {error}", attrs.node_type));
            Inline::Text(Text::default())
        }
    }
}

fn add_mark(marks: &[MarkKind], mark: MarkKind) -> Vec<MarkKind> {
    let mut marks = marks.to_vec();
    if !marks.contains(&mark) {
        marks.push(mark);
    }
    marks
}

fn marks_to_tiptap(marks: &[MarkKind]) -> Vec<Mark> {
    [MarkKind::Bold, MarkKind::Italic]
        .into_iter()
        .filter(|mark| marks.contains(mark))
        .map(|mark| {
            Mark::Known(KnownMark {
                r#type: match mark {
                    MarkKind::Bold => MarkType::Bold,
                    MarkKind::Italic => MarkType::Italic,
                },
            })
        })
        .collect()
}

fn merge_adjacent_text_nodes(nodes: Vec<InlineNode>) -> Vec<InlineNode> {
    nodes.into_iter().fold(Vec::new(), |mut merged, node| {
        if let Some(InlineNode::Text(previous)) = merged.last_mut()
            && let InlineNode::Text(ref current) = node
            && previous.marks == current.marks
        {
            previous.text.push_str(&current.text);
            return merged;
        }

        merged.push(node);
        merged
    })
}

#[allow(dead_code)]
fn _assert_value_preserves_order(_: Value) {}
