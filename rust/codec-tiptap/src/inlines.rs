//! Inline-level conversion between Tiptap JSON nodes and Stencila inlines.
//!
//! The native inline mapping currently covers text plus common text marks.
//! Unsupported Stencila inlines are preserved as custom opaque `stencilaInline`
//! nodes, while unsupported native Tiptap inlines and marks are recorded as
//! losses until explicit mappings are added.

use serde_json::Value;
use stencila_codec::stencila_schema::{
    CodeInline, Emphasis, Inline, Link, Strikeout, Strong, Subscript, Superscript, Text, Underline,
};

use crate::{
    shared::{MarkKind, TiptapDecodeContext, TiptapEncodeContext},
    tiptap::{
        self, InlineNode, KnownMark, Mark, MarkAttrs, MarkType, StencilaAttrs, StencilaInlineNode,
        TextNode,
    },
};

/// Canonical order for treating Tiptap marks as a set.
///
/// Tiptap stores marks as a flat array on each text node, while Stencila stores
/// them as nested inline nodes. Applying marks in this order gives decode a
/// stable nesting shape; emitting marks in this order gives encode canonical
/// JSON and lets adjacent text nodes with equivalent mark sets merge reliably.
const MARK_ORDER: [MarkType; 8] = [
    MarkType::Bold,
    MarkType::Italic,
    MarkType::Code,
    MarkType::Strikeout,
    MarkType::Underline,
    MarkType::Subscript,
    MarkType::Superscript,
    MarkType::Link,
];

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

        Inline::CodeInline(code) => {
            let marks = add_mark(marks, code_mark(code));
            text_value_to_tiptap(code.code.to_string(), &marks)
        }

        Inline::Link(link) => {
            let marks = add_mark(marks, link_mark(link));
            inlines_to_tiptap_with_marks(&link.content, &marks, context)
        }

        Inline::Strikeout(Strikeout { content, .. }) => {
            let marks = add_mark(marks, MarkKind::Strikeout);
            inlines_to_tiptap_with_marks(content, &marks, context)
        }

        Inline::Subscript(Subscript { content, .. }) => {
            let marks = add_mark(marks, MarkKind::Subscript);
            inlines_to_tiptap_with_marks(content, &marks, context)
        }

        Inline::Superscript(Superscript { content, .. }) => {
            let marks = add_mark(marks, MarkKind::Superscript);
            inlines_to_tiptap_with_marks(content, &marks, context)
        }

        Inline::Underline(Underline { content, .. }) => {
            let marks = add_mark(marks, MarkKind::Underline);
            inlines_to_tiptap_with_marks(content, &marks, context)
        }

        _ => vec![opaque_inline_to_tiptap(inline, context)],
    }
}

fn text_from_tiptap(value: String, marks: Vec<Mark>, context: &mut TiptapDecodeContext) -> Inline {
    let mut marks = marks_from_tiptap(marks, context);

    let mut inline = match remove_mark(&mut marks, MarkType::Code) {
        Some(MarkKind::Code(MarkAttrs {
            programming_language,
            ..
        })) => Inline::CodeInline(CodeInline {
            code: value.into(),
            programming_language,
            ..Default::default()
        }),
        _ => Inline::Text(Text::new(value.into())),
    };

    for mark_type in MARK_ORDER {
        if mark_type == MarkType::Code {
            continue;
        }

        if let Some(mark) = remove_mark(&mut marks, mark_type) {
            inline = apply_mark(inline, mark);
        }
    }

    inline
}

fn marks_from_tiptap(marks: Vec<Mark>, context: &mut TiptapDecodeContext) -> Vec<MarkKind> {
    let mut known = Vec::new();

    for mark in marks {
        match mark {
            Mark::Known(KnownMark {
                r#type: mark_type,
                attrs,
            }) => match known_mark_from_tiptap(mark_type, attrs, context) {
                Some(mark) => add_known_mark(&mut known, mark),
                None => continue,
            },
            Mark::Unknown(value) => {
                context
                    .losses
                    .add(format!("Unknown mark ({})", tiptap::value_type(&value)));
            }
        }
    }

    known
}

fn known_mark_from_tiptap(
    mark_type: MarkType,
    attrs: Option<MarkAttrs>,
    context: &mut TiptapDecodeContext,
) -> Option<MarkKind> {
    if let Some(attrs) = &attrs {
        record_unsupported_mark_attrs(mark_type, attrs, context);
    }

    match mark_type {
        MarkType::Bold => Some(MarkKind::Bold),
        MarkType::Code => Some(MarkKind::Code(attrs.unwrap_or_default())),
        MarkType::Italic => Some(MarkKind::Italic),
        MarkType::Link => {
            let Some(attrs) = attrs else {
                context.losses.add("Link.href");
                return None;
            };
            if attrs.href.is_none() {
                context.losses.add("Link.href");
                return None;
            }
            Some(MarkKind::Link(attrs))
        }
        MarkType::Strikeout => Some(MarkKind::Strikeout),
        MarkType::Subscript => Some(MarkKind::Subscript),
        MarkType::Superscript => Some(MarkKind::Superscript),
        MarkType::Underline => Some(MarkKind::Underline),
    }
}

fn record_unsupported_mark_attrs(
    mark_type: MarkType,
    attrs: &MarkAttrs,
    context: &mut TiptapDecodeContext,
) {
    let supported_attrs: &[&str] = match mark_type {
        MarkType::Code => &["programmingLanguage"],
        MarkType::Link => &["href", "title", "rel", "labelOnly"],
        MarkType::Bold
        | MarkType::Italic
        | MarkType::Strikeout
        | MarkType::Subscript
        | MarkType::Superscript
        | MarkType::Underline => &[],
    };

    for attr_name in present_mark_attr_names(attrs) {
        if !supported_attrs.contains(&attr_name) {
            context
                .losses
                .add(format!("{}.{attr_name}", mark_loss_prefix(mark_type)));
        }
    }
}

fn present_mark_attr_names(attrs: &MarkAttrs) -> Vec<&str> {
    let mut names = Vec::new();

    if attrs.href.is_some() {
        names.push("href");
    }
    if attrs.title.is_some() {
        names.push("title");
    }
    if attrs.rel.is_some() {
        names.push("rel");
    }
    if attrs.label_only.is_some() {
        names.push("labelOnly");
    }
    if attrs.programming_language.is_some() {
        names.push("programmingLanguage");
    }
    names.extend(attrs.extra.keys().map(String::as_str));

    names
}

fn mark_loss_prefix(mark_type: MarkType) -> &'static str {
    match mark_type {
        MarkType::Bold => "Strong",
        MarkType::Code => "CodeInline",
        MarkType::Italic => "Emphasis",
        MarkType::Link => "Link",
        MarkType::Strikeout => "Strikeout",
        MarkType::Subscript => "Subscript",
        MarkType::Superscript => "Superscript",
        MarkType::Underline => "Underline",
    }
}

fn add_known_mark(marks: &mut Vec<MarkKind>, mark: MarkKind) {
    if !marks.contains(&mark) {
        marks.push(mark);
    }
}

fn remove_mark(marks: &mut Vec<MarkKind>, mark_type: MarkType) -> Option<MarkKind> {
    marks
        .iter()
        .position(|mark| mark.mark_type() == mark_type)
        .map(|index| marks.remove(index))
}

fn apply_mark(inline: Inline, mark: MarkKind) -> Inline {
    match mark {
        MarkKind::Bold => Inline::Strong(Strong::new(vec![inline])),
        MarkKind::Code(..) => inline,
        MarkKind::Italic => Inline::Emphasis(Emphasis::new(vec![inline])),
        MarkKind::Link(MarkAttrs {
            href,
            title,
            rel,
            label_only,
            ..
        }) => Inline::Link(Link {
            content: vec![inline],
            target: href.unwrap_or_default(),
            title,
            rel,
            label_only,
            ..Default::default()
        }),
        MarkKind::Strikeout => Inline::Strikeout(Strikeout::new(vec![inline])),
        MarkKind::Subscript => Inline::Subscript(Subscript::new(vec![inline])),
        MarkKind::Superscript => Inline::Superscript(Superscript::new(vec![inline])),
        MarkKind::Underline => Inline::Underline(Underline::new(vec![inline])),
    }
}

fn text_to_tiptap(text: &Text, marks: &[MarkKind]) -> Vec<InlineNode> {
    text_value_to_tiptap(text.value.to_string(), marks)
}

fn text_value_to_tiptap(value: String, marks: &[MarkKind]) -> Vec<InlineNode> {
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

fn code_mark(code: &CodeInline) -> MarkKind {
    MarkKind::Code(MarkAttrs {
        programming_language: code.programming_language.clone(),
        ..Default::default()
    })
}

fn link_mark(link: &Link) -> MarkKind {
    MarkKind::Link(MarkAttrs {
        href: Some(link.target.to_string()),
        title: link.title.clone(),
        rel: link.rel.clone(),
        label_only: link.label_only,
        ..Default::default()
    })
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
    MARK_ORDER
        .into_iter()
        .filter_map(|mark_type| {
            let mark = marks.iter().find(|mark| mark.mark_type() == mark_type)?;
            Some(Mark::Known(KnownMark {
                r#type: mark_type,
                attrs: match mark {
                    MarkKind::Code(attrs) | MarkKind::Link(attrs) => {
                        if attrs.is_empty() {
                            None
                        } else {
                            Some(attrs.clone())
                        }
                    }
                    _ => None,
                },
            }))
        })
        .collect()
}

impl MarkKind {
    fn mark_type(&self) -> MarkType {
        match self {
            MarkKind::Bold => MarkType::Bold,
            MarkKind::Code(..) => MarkType::Code,
            MarkKind::Italic => MarkType::Italic,
            MarkKind::Link(..) => MarkType::Link,
            MarkKind::Strikeout => MarkType::Strikeout,
            MarkKind::Subscript => MarkType::Subscript,
            MarkKind::Superscript => MarkType::Superscript,
            MarkKind::Underline => MarkType::Underline,
        }
    }
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
