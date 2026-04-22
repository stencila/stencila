use std::collections::HashSet;

use rand::{RngExt, distr::Alphanumeric, rng};
use schema::{Node, SuggestionBlock, SuggestionInline, Visitor, VisitorMut, WalkControl, WalkNode};

/// Ensure that all suggestion nodes in a document tree have a persistent `id`.
///
/// Existing identifiers are preserved. Missing identifiers are filled with
/// short random base62 values prefixed with `sg`.
pub fn ensure_suggestion_ids(node: &mut Node) {
    let mut collector = SuggestionIdCollector::default();
    node.walk(&mut collector);

    let mut visitor = SuggestionIdEnsurer::new(collector.used_ids);
    node.walk_mut(&mut visitor);
}

#[derive(Default)]
struct SuggestionIdCollector {
    used_ids: HashSet<String>,
}

impl Visitor for SuggestionIdCollector {
    fn visit_suggestion_block(&mut self, block: &SuggestionBlock) -> WalkControl {
        if let Some(id) = &block.id {
            self.used_ids.insert(id.clone());
        }
        WalkControl::Continue
    }

    fn visit_suggestion_inline(&mut self, inline: &SuggestionInline) -> WalkControl {
        if let Some(id) = &inline.id {
            self.used_ids.insert(id.clone());
        }
        WalkControl::Continue
    }
}

struct SuggestionIdEnsurer {
    used_ids: HashSet<String>,
}

impl SuggestionIdEnsurer {
    fn new(used_ids: HashSet<String>) -> Self {
        Self { used_ids }
    }

    fn ensure_id(&mut self, id: &mut Option<String>) {
        if id.is_some() {
            return;
        }

        loop {
            let id_new = generate_suggestion_id();
            if self.used_ids.insert(id_new.clone()) {
                *id = Some(id_new);
                break;
            }
        }
    }
}

impl VisitorMut for SuggestionIdEnsurer {
    fn visit_suggestion_block(&mut self, block: &mut SuggestionBlock) -> WalkControl {
        self.ensure_id(&mut block.id);
        WalkControl::Continue
    }

    fn visit_suggestion_inline(&mut self, inline: &mut SuggestionInline) -> WalkControl {
        self.ensure_id(&mut inline.id);
        WalkControl::Continue
    }
}

fn generate_suggestion_id() -> String {
    let suffix: String = rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();

    format!("sg{suffix}")
}

#[cfg(test)]
mod tests {
    use schema::{
        Article, Block, Inline, Node, Paragraph, SuggestionBlock, SuggestionInline, SuggestionType,
        Text,
    };

    use super::ensure_suggestion_ids;

    fn text_inline(value: &str) -> Inline {
        Inline::Text(Text::from(value))
    }

    fn paragraph(content: Vec<Inline>) -> Block {
        Block::Paragraph(Paragraph::new(content))
    }

    fn insert_inline(content: Vec<Inline>) -> Inline {
        Inline::SuggestionInline(SuggestionInline {
            suggestion_type: Some(SuggestionType::Insert),
            content,
            ..Default::default()
        })
    }

    fn insert_block(content: Vec<Block>) -> Block {
        Block::SuggestionBlock(SuggestionBlock {
            suggestion_type: Some(SuggestionType::Insert),
            content,
            ..Default::default()
        })
    }

    #[test]
    fn assigns_missing_ids() {
        let mut node = Node::Article(Article::new(vec![Block::Paragraph(Paragraph::new(vec![
            text_inline("before "),
            insert_inline(vec![text_inline("inline")]),
            text_inline(" after"),
        ]))]));

        if let Node::Article(article) = &mut node {
            article
                .content
                .push(insert_block(vec![paragraph(vec![text_inline("block")])]));
        }

        ensure_suggestion_ids(&mut node);

        let Node::Article(article) = node else {
            panic!("expected Article");
        };

        let Block::Paragraph(paragraph) = &article.content[0] else {
            panic!("expected Paragraph");
        };
        let Inline::SuggestionInline(inline) = &paragraph.content[1] else {
            panic!("expected SuggestionInline");
        };
        let Block::SuggestionBlock(block) = &article.content[1] else {
            panic!("expected SuggestionBlock");
        };

        let Some(inline_id) = &inline.id else {
            panic!("expected suggestion inline id");
        };
        let Some(block_id) = &block.id else {
            panic!("expected suggestion block id");
        };

        assert!(inline_id.starts_with("sg"));
        assert!(block_id.starts_with("sg"));
        assert_ne!(inline_id, block_id);
    }

    #[test]
    fn preserves_existing_ids() {
        let mut node = Node::Article(Article::new(vec![Block::SuggestionBlock(
            SuggestionBlock {
                id: Some("suggestion-1".into()),
                suggestion_type: Some(SuggestionType::Insert),
                content: vec![paragraph(vec![text_inline("block")])],
                ..Default::default()
            },
        )]));

        ensure_suggestion_ids(&mut node);

        let Node::Article(article) = node else {
            panic!("expected Article");
        };
        let Block::SuggestionBlock(block) = &article.content[0] else {
            panic!("expected SuggestionBlock");
        };

        assert_eq!(block.id.as_deref(), Some("suggestion-1"));
    }
}
