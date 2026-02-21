//! Wait-for-human handler (§4.6).
//!
//! Blocks pipeline execution until a human selects an option derived
//! from the node's outgoing edges. Implements the human-in-the-loop
//! pattern using the [`Interviewer`] trait.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use indexmap::IndexMap;

use crate::context::Context;
use crate::error::AttractorResult;
use crate::events::{EventEmitter, NoOpEmitter, PipelineEvent};
use crate::graph::{Graph, Node};
use crate::handler::Handler;
use crate::interviewer::{Answer, AnswerValue, Interviewer, Question, QuestionOption};
use crate::types::Outcome;

/// Handler for `wait.human` nodes that presents choices to a human.
///
/// Choices are derived from the node's outgoing edges. The selected
/// choice determines which edge to follow via `suggested_next_ids`.
pub struct WaitForHumanHandler {
    interviewer: Arc<dyn Interviewer>,
    emitter: Arc<dyn EventEmitter>,
}

impl std::fmt::Debug for WaitForHumanHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WaitForHumanHandler")
            .finish_non_exhaustive()
    }
}

impl WaitForHumanHandler {
    /// Create a new handler with the given interviewer and event emitter.
    #[must_use]
    pub fn with_emitter(interviewer: Arc<dyn Interviewer>, emitter: Arc<dyn EventEmitter>) -> Self {
        Self {
            interviewer,
            emitter,
        }
    }

    /// Create a new handler with the given interviewer and a no-op emitter.
    #[must_use]
    pub fn new(interviewer: Arc<dyn Interviewer>) -> Self {
        Self {
            interviewer,
            emitter: Arc::new(NoOpEmitter),
        }
    }
}

/// A choice derived from an outgoing edge.
#[derive(Debug, Clone)]
struct Choice {
    key: String,
    label: String,
    target: String,
}

/// Parse an accelerator key from an edge label (§4.6).
///
/// Supported patterns:
/// - `[K] Label` → `K`
/// - `K) Label` → `K`
/// - `K - Label` → `K`
/// - `Label` → first character uppercased
#[must_use]
pub fn parse_accelerator_key(label: &str) -> String {
    let trimmed = label.trim();

    // Pattern: [K] Label
    if let Some(rest) = trimmed.strip_prefix('[')
        && let Some(bracket_end) = rest.find(']')
    {
        let key = &rest[..bracket_end];
        if !key.is_empty() {
            return key.to_uppercase();
        }
    }

    // Pattern: K) Label
    if trimmed.len() >= 2 {
        let bytes = trimmed.as_bytes();
        if bytes.get(1) == Some(&b')') {
            return trimmed[..1].to_uppercase();
        }
    }

    // Pattern: K - Label
    if trimmed.len() >= 4 && trimmed.as_bytes().get(1..4) == Some(b" - ".as_slice()) {
        return trimmed[..1].to_uppercase();
    }

    // Fallback: first character
    trimmed
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_default()
}

#[async_trait]
impl Handler for WaitForHumanHandler {
    async fn execute(
        &self,
        node: &Node,
        _context: &Context,
        graph: &Graph,
        _logs_root: &Path,
    ) -> AttractorResult<Outcome> {
        // 1. Derive choices from outgoing edges
        let edges = graph.outgoing_edges(&node.id);
        let choices: Vec<Choice> = edges
            .iter()
            .map(|edge| {
                let label = if edge.label().is_empty() {
                    edge.to.clone()
                } else {
                    edge.label().to_string()
                };
                let key = parse_accelerator_key(&label);
                Choice {
                    key,
                    label,
                    target: edge.to.clone(),
                }
            })
            .collect();

        if choices.is_empty() {
            return Ok(Outcome::fail("No outgoing edges for human gate"));
        }

        // 2. Build question from choices
        let options: Vec<QuestionOption> = choices
            .iter()
            .map(|c| QuestionOption {
                key: c.key.clone(),
                label: c.label.clone(),
            })
            .collect();

        let text = node.get_str_attr("label").unwrap_or("Select an option:");
        let mut question = Question::multiple_choice(text, options, &node.id);

        // Set timeout from node attribute (§2.6)
        if let Some(v) = node.get_attr("timeout") {
            let secs = match v {
                crate::graph::AttrValue::Duration(d) => Some(d.inner().as_secs_f64()),
                crate::graph::AttrValue::String(s) => crate::types::Duration::from_spec_str(s)
                    .ok()
                    .map(|d| d.inner().as_secs_f64()),
                _ => None,
            };
            question.timeout_seconds = secs;
        }

        // 3. Present to interviewer and wait
        self.emitter.emit(PipelineEvent::InterviewQuestionAsked {
            node_id: node.id.clone(),
        });
        let answer = self.interviewer.ask(&question).await;

        // 4. Handle timeout/skip
        if answer.is_timeout() {
            self.emitter.emit(PipelineEvent::InterviewTimedOut {
                node_id: node.id.clone(),
            });

            if let Some(default_target) = node.get_str_attr("human.default_choice")
                && let Some(choice) = find_choice_by_str(default_target, &choices)
            {
                return Ok(build_human_outcome(choice));
            }
            // default_choice absent or doesn't match any known edge
            // target — fall through to the retry outcome below.
            return Ok(Outcome::retry("human gate timeout, no default"));
        }

        if answer.is_skipped() {
            return Ok(Outcome::fail("human skipped interaction"));
        }

        self.emitter.emit(PipelineEvent::InterviewAnswerReceived {
            node_id: node.id.clone(),
        });

        // 5. Find matching choice — no silent fallback to choices[0] so
        //    invalid answers don't silently route to an unintended branch.
        let Some(selected) = find_matching_choice(&answer, &choices) else {
            return Ok(Outcome::fail("answer did not match any available choice"));
        };

        // 6. Build outcome
        Ok(build_human_outcome(selected))
    }
}

/// Build a success outcome for a selected choice.
fn build_human_outcome(choice: &Choice) -> Outcome {
    let mut updates = IndexMap::new();
    updates.insert(
        "human.gate.selected".into(),
        serde_json::Value::String(choice.key.clone()),
    );
    updates.insert(
        "human.gate.label".into(),
        serde_json::Value::String(choice.label.clone()),
    );

    Outcome {
        status: crate::types::StageStatus::Success,
        suggested_next_ids: vec![choice.target.clone()],
        context_updates: updates,
        ..Outcome::success()
    }
}

/// Find a choice by a string value, checking target, key, and label.
///
/// Resolution order: exact target match, case-insensitive key, then
/// case-insensitive label. Used for `human.default_choice` resolution.
fn find_choice_by_str<'a>(text: &str, choices: &'a [Choice]) -> Option<&'a Choice> {
    choices
        .iter()
        .find(|c| c.target == text)
        .or_else(|| choices.iter().find(|c| c.key.eq_ignore_ascii_case(text)))
        .or_else(|| choices.iter().find(|c| c.label.eq_ignore_ascii_case(text)))
}

/// Find the choice matching an answer.
///
/// Matching strategy (deviation from §4.6 pseudocode):
/// - `Selected(key)` → exact key match
/// - `Text(text)` → case-insensitive match by key, then by label
/// - `Yes` → first choice (convenience for single-choice gates)
/// - All other variants → no match (returns `None`)
///
/// The spec pseudocode matches only by key. We extend with label
/// matching to improve UX when humans type full labels.
fn find_matching_choice<'a>(answer: &Answer, choices: &'a [Choice]) -> Option<&'a Choice> {
    match &answer.value {
        AnswerValue::Selected(key) => choices.iter().find(|c| c.key.eq_ignore_ascii_case(key)),
        AnswerValue::Text(text) => {
            // Try matching by key first, then by label
            choices
                .iter()
                .find(|c| c.key.eq_ignore_ascii_case(text))
                .or_else(|| choices.iter().find(|c| c.label.eq_ignore_ascii_case(text)))
        }
        AnswerValue::Yes => choices.first(),
        _ => None,
    }
}

// Note: `wait.human` is not included in `HandlerRegistry::with_defaults()`
// because it requires an `Interviewer` instance. Callers must register it
// explicitly — see `parse_accelerator_key` which is public for testing.
