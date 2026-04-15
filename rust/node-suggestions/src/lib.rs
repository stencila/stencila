#![warn(clippy::pedantic)]

use std::collections::HashMap;

use schema::{Block, Inline, Node, NodeId, SuggestionBlock, SuggestionInline, SuggestionType};

pub mod review;

/// The action to take when resolving suggestions
#[derive(Debug, Clone)]
pub enum SuggestionAction {
    /// Accept all suggestions
    AcceptAll,
    /// Reject all suggestions
    RejectAll,
    /// Per-suggestion decisions keyed by node id
    Review(HashMap<NodeId, SingleAction>),
}

/// Action for a single suggestion during interactive review
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SingleAction {
    Accept,
    Reject,
    Skip,
}

/// Resolve suggestions in a node tree by accepting, rejecting, or reviewing them
pub trait ResolveSuggestions {
    fn resolve_suggestions(&mut self, action: &SuggestionAction);
}

impl ResolveSuggestions for Node {
    fn resolve_suggestions(&mut self, action: &SuggestionAction) {
        if let Node::Article(article) = self {
            article.content.resolve_suggestions(action);
        }
    }
}

impl ResolveSuggestions for Vec<Block> {
    fn resolve_suggestions(&mut self, action: &SuggestionAction) {
        let blocks = std::mem::take(self);
        *self = blocks
            .into_iter()
            .flat_map(|block| resolve_block(block, action))
            .collect();
    }
}

/// Resolve a single block, returning zero or more blocks
fn resolve_block(mut block: Block, action: &SuggestionAction) -> Vec<Block> {
    if let Block::SuggestionBlock(sb) = block {
        resolve_suggestion_block(sb, action)
    } else {
        // Recurse into nested content first
        block.resolve_suggestions(action);
        vec![block]
    }
}

/// Resolve a `SuggestionBlock` based on its type and the action
fn resolve_suggestion_block(mut sb: SuggestionBlock, action: &SuggestionAction) -> Vec<Block> {
    let single = match action {
        SuggestionAction::AcceptAll => SingleAction::Accept,
        SuggestionAction::RejectAll => SingleAction::Reject,
        SuggestionAction::Review(map) => map
            .get(&sb.node_id())
            .copied()
            .unwrap_or(SingleAction::Skip),
    };

    if single == SingleAction::Skip {
        // Recurse into suggestion content but leave the node in place
        sb.content.resolve_suggestions(action);
        return vec![Block::SuggestionBlock(sb)];
    }

    let is_insert = sb
        .suggestion_type
        .as_ref()
        .is_none_or(|t| *t == SuggestionType::Insert);

    let is_replace = sb.suggestion_type == Some(SuggestionType::Replace);

    let keep_content = match (single, sb.suggestion_type) {
        (SingleAction::Accept, Some(SuggestionType::Delete))
        | (SingleAction::Reject, Some(SuggestionType::Replace)) => false,
        (SingleAction::Reject, Some(SuggestionType::Delete))
        | (SingleAction::Accept, Some(SuggestionType::Replace)) => true,
        (SingleAction::Accept, _) => is_insert,
        (SingleAction::Reject, _) => !is_insert,
        (SingleAction::Skip, _) => unreachable!("handled above"),
    };

    if keep_content {
        let mut content = sb.content;
        content.resolve_suggestions(action);
        content
    } else if is_replace {
        let mut original = sb.original.unwrap_or_default();
        original.resolve_suggestions(action);
        original
    } else {
        vec![]
    }
}

impl ResolveSuggestions for Vec<Inline> {
    fn resolve_suggestions(&mut self, action: &SuggestionAction) {
        let inlines = std::mem::take(self);
        *self = inlines
            .into_iter()
            .flat_map(|inline| resolve_inline(inline, action))
            .collect();
    }
}

/// Resolve a single inline, returning zero or more inlines
fn resolve_inline(mut inline: Inline, action: &SuggestionAction) -> Vec<Inline> {
    if let Inline::SuggestionInline(si) = inline {
        resolve_suggestion_inline(si, action)
    } else {
        inline.resolve_suggestions(action);
        vec![inline]
    }
}

/// Resolve a `SuggestionInline` based on its type and the action
fn resolve_suggestion_inline(mut si: SuggestionInline, action: &SuggestionAction) -> Vec<Inline> {
    let single = match action {
        SuggestionAction::AcceptAll => SingleAction::Accept,
        SuggestionAction::RejectAll => SingleAction::Reject,
        SuggestionAction::Review(map) => map
            .get(&si.node_id())
            .copied()
            .unwrap_or(SingleAction::Skip),
    };

    if single == SingleAction::Skip {
        si.content.resolve_suggestions(action);
        return vec![Inline::SuggestionInline(si)];
    }

    let is_insert = si
        .suggestion_type
        .as_ref()
        .is_none_or(|t| *t == SuggestionType::Insert);

    let is_replace = si.suggestion_type == Some(SuggestionType::Replace);

    let keep_content = match (single, si.suggestion_type) {
        (SingleAction::Accept, Some(SuggestionType::Delete))
        | (SingleAction::Reject, Some(SuggestionType::Replace)) => false,
        (SingleAction::Reject, Some(SuggestionType::Delete))
        | (SingleAction::Accept, Some(SuggestionType::Replace)) => true,
        (SingleAction::Accept, _) => is_insert,
        (SingleAction::Reject, _) => !is_insert,
        (SingleAction::Skip, _) => unreachable!("handled above"),
    };

    if keep_content {
        let mut content = si.content;
        content.resolve_suggestions(action);
        content
    } else if is_replace {
        let mut original = si.original.unwrap_or_default();
        original.resolve_suggestions(action);
        original
    } else {
        vec![]
    }
}

// -- Recursion into Block variants --

impl ResolveSuggestions for Block {
    fn resolve_suggestions(&mut self, action: &SuggestionAction) {
        match self {
            Block::Admonition(n) => {
                if let Some(title) = &mut n.title {
                    title.resolve_suggestions(action);
                }
                n.content.resolve_suggestions(action);
            }
            Block::Chat(n) => n.content.resolve_suggestions(action),
            Block::ChatMessage(n) => n.content.resolve_suggestions(action),
            Block::ChatMessageGroup(n) => {
                for msg in &mut n.messages {
                    msg.content.resolve_suggestions(action);
                }
            }
            Block::Claim(n) => n.content.resolve_suggestions(action),
            Block::Excerpt(n) => n.content.resolve_suggestions(action),
            Block::Figure(n) => {
                if let Some(caption) = &mut n.caption {
                    caption.resolve_suggestions(action);
                }
                n.content.resolve_suggestions(action);
            }
            Block::ForBlock(n) => {
                n.content.resolve_suggestions(action);
                if let Some(otherwise) = &mut n.otherwise {
                    otherwise.resolve_suggestions(action);
                }
            }
            Block::Form(n) => n.content.resolve_suggestions(action),
            Block::Heading(n) => n.content.resolve_suggestions(action),
            Block::IfBlock(n) => {
                for clause in &mut n.clauses {
                    clause.content.resolve_suggestions(action);
                }
            }
            Block::Island(n) => n.content.resolve_suggestions(action),
            Block::List(n) => {
                for item in &mut n.items {
                    item.content.resolve_suggestions(action);
                }
            }
            Block::Page(n) => n.content.resolve_suggestions(action),
            Block::Paragraph(n) => n.content.resolve_suggestions(action),
            Block::QuoteBlock(n) => n.content.resolve_suggestions(action),
            Block::Section(n) => n.content.resolve_suggestions(action),
            Block::StyledBlock(n) => n.content.resolve_suggestions(action),
            Block::SuggestionBlock(n) => {
                // Recurse into nested content (the flatmap in Vec<Block> handles the suggestion itself)
                if let Some(original) = &mut n.original {
                    original.resolve_suggestions(action);
                }
                n.content.resolve_suggestions(action);
            }
            Block::Table(n) => {
                if let Some(caption) = &mut n.caption {
                    caption.resolve_suggestions(action);
                }
                for row in &mut n.rows {
                    for cell in &mut row.cells {
                        cell.content.resolve_suggestions(action);
                    }
                }
                if let Some(notes) = &mut n.notes {
                    notes.resolve_suggestions(action);
                }
            }
            Block::Walkthrough(n) => {
                for step in &mut n.steps {
                    step.content.resolve_suggestions(action);
                }
            }
            // Leaf block types with no nested block/inline content
            Block::AppendixBreak(_)
            | Block::AudioObject(_)
            | Block::CallBlock(_)
            | Block::CodeBlock(_)
            | Block::CodeChunk(_)
            | Block::Datatable(_)
            | Block::File(_)
            | Block::ImageObject(_)
            | Block::IncludeBlock(_)
            | Block::InlinesBlock(_)
            | Block::InstructionBlock(_)
            | Block::MathBlock(_)
            | Block::PromptBlock(_)
            | Block::RawBlock(_)
            | Block::Supplement(_)
            | Block::ThematicBreak(_)
            | Block::VideoObject(_) => {}
        }
    }
}

// -- Recursion into Inline variants --

impl ResolveSuggestions for Inline {
    fn resolve_suggestions(&mut self, action: &SuggestionAction) {
        match self {
            Inline::Annotation(n) => n.content.resolve_suggestions(action),
            Inline::Emphasis(n) => n.content.resolve_suggestions(action),
            Inline::Link(n) => n.content.resolve_suggestions(action),
            Inline::Note(n) => n.content.resolve_suggestions(action),
            Inline::QuoteInline(n) => n.content.resolve_suggestions(action),
            Inline::Sentence(n) => n.content.resolve_suggestions(action),
            Inline::Strikeout(n) => n.content.resolve_suggestions(action),
            Inline::Strong(n) => n.content.resolve_suggestions(action),
            Inline::StyledInline(n) => n.content.resolve_suggestions(action),
            Inline::Subscript(n) => n.content.resolve_suggestions(action),
            Inline::SuggestionInline(n) => {
                // Recurse into nested content (the flatmap in Vec<Inline> handles the suggestion itself)
                if let Some(original) = &mut n.original {
                    original.resolve_suggestions(action);
                }
                n.content.resolve_suggestions(action);
            }
            Inline::Superscript(n) => n.content.resolve_suggestions(action),
            Inline::Underline(n) => n.content.resolve_suggestions(action),
            // Leaf inline types with no nested inline content
            Inline::AudioObject(_)
            | Inline::Boolean(_)
            | Inline::Boundary(_)
            | Inline::Button(_)
            | Inline::Citation(_)
            | Inline::CitationGroup(_)
            | Inline::CodeExpression(_)
            | Inline::CodeInline(_)
            | Inline::Date(_)
            | Inline::DateTime(_)
            | Inline::Duration(_)
            | Inline::Icon(_)
            | Inline::ImageObject(_)
            | Inline::InstructionInline(_)
            | Inline::Integer(_)
            | Inline::MathInline(_)
            | Inline::MediaObject(_)
            | Inline::Null(_)
            | Inline::Number(_)
            | Inline::Parameter(_)
            | Inline::Text(_)
            | Inline::Time(_)
            | Inline::Timestamp(_)
            | Inline::UnsignedInteger(_)
            | Inline::VideoObject(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use schema::{Paragraph, Section, Text};

    use super::*;

    fn text_inline(s: &str) -> Inline {
        Inline::Text(Text::from(s))
    }

    fn para(inlines: Vec<Inline>) -> Block {
        Block::Paragraph(Paragraph::new(inlines))
    }

    fn insert_block(content: Vec<Block>) -> Block {
        Block::SuggestionBlock(SuggestionBlock {
            suggestion_type: Some(SuggestionType::Insert),
            content,
            ..Default::default()
        })
    }

    fn delete_block(content: Vec<Block>) -> Block {
        Block::SuggestionBlock(SuggestionBlock {
            suggestion_type: Some(SuggestionType::Delete),
            content,
            ..Default::default()
        })
    }

    fn insert_inline(content: Vec<Inline>) -> Inline {
        Inline::SuggestionInline(SuggestionInline {
            suggestion_type: Some(SuggestionType::Insert),
            content,
            ..Default::default()
        })
    }

    fn delete_inline(content: Vec<Inline>) -> Inline {
        Inline::SuggestionInline(SuggestionInline {
            suggestion_type: Some(SuggestionType::Delete),
            content,
            ..Default::default()
        })
    }

    fn replace_block(original: Vec<Block>, content: Vec<Block>) -> Block {
        Block::SuggestionBlock(SuggestionBlock {
            suggestion_type: Some(SuggestionType::Replace),
            content,
            original: Some(original),
            ..Default::default()
        })
    }

    fn replace_inline(original: Vec<Inline>, content: Vec<Inline>) -> Inline {
        Inline::SuggestionInline(SuggestionInline {
            suggestion_type: Some(SuggestionType::Replace),
            content,
            original: Some(original),
            ..Default::default()
        })
    }

    #[test]
    fn accept_insert_block_splices_content() {
        let mut blocks = vec![
            para(vec![text_inline("before")]),
            insert_block(vec![para(vec![text_inline("inserted")])]),
            para(vec![text_inline("after")]),
        ];
        blocks.resolve_suggestions(&SuggestionAction::AcceptAll);
        assert_eq!(blocks.len(), 3);
        // The suggestion is replaced by its content paragraph
        assert!(matches!(&blocks[1], Block::Paragraph(_)));
    }

    #[test]
    fn accept_delete_block_removes() {
        let mut blocks = vec![
            para(vec![text_inline("before")]),
            delete_block(vec![para(vec![text_inline("deleted")])]),
            para(vec![text_inline("after")]),
        ];
        blocks.resolve_suggestions(&SuggestionAction::AcceptAll);
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn reject_insert_block_removes() {
        let mut blocks = vec![
            para(vec![text_inline("before")]),
            insert_block(vec![para(vec![text_inline("inserted")])]),
            para(vec![text_inline("after")]),
        ];
        blocks.resolve_suggestions(&SuggestionAction::RejectAll);
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn reject_delete_block_splices_content() {
        let mut blocks = vec![
            para(vec![text_inline("before")]),
            delete_block(vec![para(vec![text_inline("kept")])]),
            para(vec![text_inline("after")]),
        ];
        blocks.resolve_suggestions(&SuggestionAction::RejectAll);
        assert_eq!(blocks.len(), 3);
        assert!(matches!(&blocks[1], Block::Paragraph(_)));
    }

    #[test]
    fn accept_insert_inline_splices_content() {
        let mut inlines = vec![
            text_inline("hello "),
            insert_inline(vec![text_inline("world")]),
            text_inline("!"),
        ];
        inlines.resolve_suggestions(&SuggestionAction::AcceptAll);
        assert_eq!(inlines.len(), 3);
        assert!(matches!(&inlines[1], Inline::Text(_)));
    }

    #[test]
    fn accept_delete_inline_removes() {
        let mut inlines = vec![
            text_inline("hello "),
            delete_inline(vec![text_inline("world")]),
            text_inline("!"),
        ];
        inlines.resolve_suggestions(&SuggestionAction::AcceptAll);
        assert_eq!(inlines.len(), 2);
    }

    #[test]
    fn reject_insert_inline_removes() {
        let mut inlines = vec![
            text_inline("hello "),
            insert_inline(vec![text_inline("world")]),
            text_inline("!"),
        ];
        inlines.resolve_suggestions(&SuggestionAction::RejectAll);
        assert_eq!(inlines.len(), 2);
    }

    #[test]
    fn reject_delete_inline_splices_content() {
        let mut inlines = vec![
            text_inline("hello "),
            delete_inline(vec![text_inline("world")]),
            text_inline("!"),
        ];
        inlines.resolve_suggestions(&SuggestionAction::RejectAll);
        assert_eq!(inlines.len(), 3);
    }

    #[test]
    fn nested_suggestions_in_section() {
        let mut blocks = vec![Block::Section(Section {
            content: vec![
                para(vec![text_inline("before")]),
                insert_block(vec![para(vec![text_inline("nested insert")])]),
            ],
            ..Default::default()
        })];
        blocks.resolve_suggestions(&SuggestionAction::AcceptAll);
        if let Block::Section(section) = &blocks[0] {
            assert_eq!(section.content.len(), 2);
            assert!(matches!(&section.content[1], Block::Paragraph(_)));
        } else {
            panic!("expected Section");
        }
    }

    #[test]
    fn inline_suggestion_in_paragraph() {
        let mut blocks = vec![para(vec![
            text_inline("hello "),
            insert_inline(vec![text_inline("world")]),
        ])];
        blocks.resolve_suggestions(&SuggestionAction::AcceptAll);
        if let Block::Paragraph(p) = &blocks[0] {
            assert_eq!(p.content.len(), 2);
            assert!(matches!(&p.content[1], Inline::Text(_)));
        } else {
            panic!("expected Paragraph");
        }
    }

    #[test]
    fn review_with_skip_leaves_suggestion() {
        let mut blocks = vec![insert_block(vec![para(vec![text_inline("content")])])];
        let map = HashMap::new(); // empty map means all skip
        blocks.resolve_suggestions(&SuggestionAction::Review(map));
        assert_eq!(blocks.len(), 1);
        assert!(matches!(&blocks[0], Block::SuggestionBlock(_)));
    }

    #[test]
    fn review_with_node_id_accept() {
        let sb = SuggestionBlock {
            suggestion_type: Some(SuggestionType::Insert),
            content: vec![para(vec![text_inline("inserted")])],
            ..Default::default()
        };
        let node_id = sb.node_id();
        let mut blocks = vec![Block::SuggestionBlock(sb)];
        let mut map = HashMap::new();
        map.insert(node_id, SingleAction::Accept);
        blocks.resolve_suggestions(&SuggestionAction::Review(map));
        assert_eq!(blocks.len(), 1);
        assert!(matches!(&blocks[0], Block::Paragraph(_)));
    }

    #[test]
    fn default_suggestion_type_is_insert() {
        // When suggestion_type is None, should default to Insert behavior
        let mut blocks = vec![Block::SuggestionBlock(SuggestionBlock {
            suggestion_type: None,
            content: vec![para(vec![text_inline("content")])],
            ..Default::default()
        })];
        blocks.resolve_suggestions(&SuggestionAction::AcceptAll);
        assert_eq!(blocks.len(), 1);
        assert!(matches!(&blocks[0], Block::Paragraph(_)));
    }

    #[test]
    fn multiple_blocks_from_single_suggestion() {
        let mut blocks = vec![insert_block(vec![
            para(vec![text_inline("first")]),
            para(vec![text_inline("second")]),
        ])];
        blocks.resolve_suggestions(&SuggestionAction::AcceptAll);
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn accept_replace_block_splices_new_content() {
        let mut blocks = vec![replace_block(
            vec![para(vec![text_inline("old")])],
            vec![para(vec![text_inline("new")])],
        )];
        blocks.resolve_suggestions(&SuggestionAction::AcceptAll);
        assert_eq!(blocks.len(), 1);
        let Block::Paragraph(paragraph) = &blocks[0] else {
            panic!("expected Paragraph")
        };
        assert_eq!(paragraph.content, vec![text_inline("new")]);
    }

    #[test]
    fn reject_replace_block_splices_original_content() {
        let mut blocks = vec![replace_block(
            vec![para(vec![text_inline("old")])],
            vec![para(vec![text_inline("new")])],
        )];
        blocks.resolve_suggestions(&SuggestionAction::RejectAll);
        assert_eq!(blocks.len(), 1);
        let Block::Paragraph(paragraph) = &blocks[0] else {
            panic!("expected Paragraph")
        };
        assert_eq!(paragraph.content, vec![text_inline("old")]);
    }

    #[test]
    fn accept_replace_inline_splices_new_content() {
        let mut inlines = vec![replace_inline(
            vec![text_inline("old")],
            vec![text_inline("new")],
        )];
        inlines.resolve_suggestions(&SuggestionAction::AcceptAll);
        assert_eq!(inlines, vec![text_inline("new")]);
    }

    #[test]
    fn reject_replace_inline_splices_original_content() {
        let mut inlines = vec![replace_inline(
            vec![text_inline("old")],
            vec![text_inline("new")],
        )];
        inlines.resolve_suggestions(&SuggestionAction::RejectAll);
        assert_eq!(inlines, vec![text_inline("old")]);
    }
}
