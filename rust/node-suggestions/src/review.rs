use std::collections::HashMap;

use eyre::Result;
use schema::{
    Block, Inline, Node, NodeId, SuggestionBlock, SuggestionInline, SuggestionType, Visitor,
    WalkControl, WalkNode,
};

use crate::{SingleAction, SuggestionAction};

/// Information about a suggestion found during tree walking
struct SuggestionInfo {
    /// The `node_id` of the suggestion
    node_id: NodeId,
    suggestion_type: SuggestionType,
    preview: String,
}

/// Interactively review all suggestions in a node, prompting the user for each one.
///
/// Uses `node_id()` to key suggestions, so no ID assignment is needed.
///
/// Uses the `ask` crate for prompting, so it works in both CLI and LSP contexts.
///
/// # Errors
///
/// Returns an error if prompting the user fails.
pub async fn interactive_review(node: &Node) -> Result<SuggestionAction> {
    let mut collector = SuggestionCollector::default();
    node.walk(&mut collector);
    let suggestions = collector.suggestions;

    if suggestions.is_empty() {
        return Ok(SuggestionAction::Review(HashMap::new()));
    }

    let items = vec![
        "Accept".to_string(),
        "Reject".to_string(),
        "Skip".to_string(),
    ];

    let mut decisions = HashMap::new();

    for info in suggestions {
        let type_label = match info.suggestion_type {
            SuggestionType::Insert => "insertion",
            SuggestionType::Delete => "deletion",
            SuggestionType::Replace => "replacement",
        };

        let prompt = format!("Suggested {}: {}", type_label, info.preview);

        let selection = stencila_ask::select_with_default(&prompt, &items, 2).await?;

        let action = match selection {
            0 => SingleAction::Accept,
            1 => SingleAction::Reject,
            _ => SingleAction::Skip,
        };

        decisions.insert(info.node_id, action);
    }

    Ok(SuggestionAction::Review(decisions))
}

/// A `Visitor` that collects suggestion info from the node tree
#[derive(Default)]
struct SuggestionCollector {
    suggestions: Vec<SuggestionInfo>,
}

impl Visitor for SuggestionCollector {
    fn visit_suggestion_block(&mut self, sb: &SuggestionBlock) -> WalkControl {
        self.suggestions.push(SuggestionInfo {
            node_id: sb.node_id(),
            suggestion_type: sb.suggestion_type.unwrap_or(SuggestionType::Insert),
            preview: preview_blocks(&sb.content),
        });
        WalkControl::Continue
    }

    fn visit_suggestion_inline(&mut self, si: &SuggestionInline) -> WalkControl {
        self.suggestions.push(SuggestionInfo {
            node_id: si.node_id(),
            suggestion_type: si.suggestion_type.unwrap_or(SuggestionType::Insert),
            preview: preview_inlines(&si.content),
        });
        WalkControl::Continue
    }
}

/// Generate a text preview of block content (first ~200 chars)
fn preview_blocks(blocks: &[Block]) -> String {
    let mut text = String::new();
    for block in blocks {
        if text.len() >= 200 {
            break;
        }
        append_block_text(block, &mut text);
    }
    if text.len() > 200 {
        text.truncate(200);
        text.push_str("...");
    }
    if text.is_empty() {
        "(empty)".to_string()
    } else {
        text
    }
}

fn preview_inlines(inlines: &[Inline]) -> String {
    let mut text = String::new();
    for inline in inlines {
        append_inline_text(inline, &mut text);
    }
    if text.len() > 200 {
        text.truncate(200);
        text.push_str("...");
    }
    if text.is_empty() {
        "(empty)".to_string()
    } else {
        text
    }
}

fn append_block_text(block: &Block, text: &mut String) {
    match block {
        Block::Paragraph(p) => {
            for inline in &p.content {
                append_inline_text(inline, text);
            }
            text.push('\n');
        }
        Block::Heading(h) => {
            for inline in &h.content {
                append_inline_text(inline, text);
            }
            text.push('\n');
        }
        Block::CodeBlock(cb) => {
            text.push_str(&cb.code);
            text.push('\n');
        }
        _ => {
            text.push_str("[block content]");
            text.push('\n');
        }
    }
}

fn append_inline_text(inline: &Inline, text: &mut String) {
    match inline {
        Inline::Text(t) => text.push_str(&t.value),
        Inline::Emphasis(e) => {
            for i in &e.content {
                append_inline_text(i, text);
            }
        }
        Inline::Strong(s) => {
            for i in &s.content {
                append_inline_text(i, text);
            }
        }
        Inline::CodeInline(c) => text.push_str(&c.code),
        _ => text.push_str("[...]"),
    }
}
