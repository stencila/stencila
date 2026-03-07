//! Wait-for-human handler (§4.6).
//!
//! Blocks pipeline execution until a human selects an option derived
//! from the node's outgoing edges. Implements the human-in-the-loop
//! pattern using the [`Interviewer`] trait.

use std::sync::Arc;

use async_trait::async_trait;
use indexmap::IndexMap;

use crate::context::Context;
use crate::error::AttractorResult;
use crate::events::{EventEmitter, NoOpEmitter, PipelineEvent};
use crate::graph::{Graph, Node};
use crate::handler::Handler;
use crate::interviewer::{
    Answer, AnswerValue, Interview, InterviewError, Interviewer, Question, QuestionOption,
};
use crate::types::Outcome;

/// Handler for `wait.human` nodes that presents choices to a human.
///
/// Choices are derived from the node's outgoing edges. The selected
/// choice determines which edge to follow via `suggested_next_ids`.
///
/// When a `db_conn` is provided (via [`with_db`](Self::with_db)) and the
/// handler detects a resume scenario, it queries for a pending interview
/// from a previous run and reuses its ID so that external systems (web UI,
/// Slack) that already have the question can still submit answers.
pub struct WaitForHumanHandler {
    interviewer: Arc<dyn Interviewer>,
    emitter: Arc<dyn EventEmitter>,
    db_conn: Option<Arc<std::sync::Mutex<stencila_db::rusqlite::Connection>>>,
    context_type: String,
    context_id: String,
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
            db_conn: None,
            context_type: String::new(),
            context_id: String::new(),
        }
    }

    /// Create a new handler with the given interviewer and a no-op emitter.
    #[must_use]
    pub fn new(interviewer: Arc<dyn Interviewer>) -> Self {
        Self {
            interviewer,
            emitter: Arc::new(NoOpEmitter),
            db_conn: None,
            context_type: String::new(),
            context_id: String::new(),
        }
    }

    /// Set the DB connection and context for resume-aware pending interview recovery.
    ///
    /// When set, the handler queries for pending interviews from a previous
    /// run before creating a new one, enabling crash recovery for in-flight
    /// human gates.
    #[must_use]
    pub fn with_db(
        mut self,
        db_conn: Arc<std::sync::Mutex<stencila_db::rusqlite::Connection>>,
        context_type: impl Into<String>,
        context_id: impl Into<String>,
    ) -> Self {
        self.db_conn = Some(db_conn);
        self.context_type = context_type.into();
        self.context_id = context_id.into();
        self
    }
}

/// A choice derived from an outgoing edge.
#[derive(Debug, Clone)]
struct Choice {
    key: String,
    label: String,
    target: String,
}

/// Parse an accelerator key and display label from an edge label (§4.6).
///
/// Returns `(key, display_label)` where the key is the accelerator
/// (uppercased) and the display label has the prefix stripped.
///
/// Supported patterns:
/// - `[K] Label` → `("K", "Label")`
/// - `K) Label`  → `("K", "Label")`
/// - `K - Label` → `("K", "Label")`
/// - `Label`     → `("L", "Label")` (first character uppercased)
#[must_use]
pub fn parse_accelerator_label(label: &str) -> (String, String) {
    let trimmed = label.trim();

    // Pattern: [K] Label
    if let Some(rest) = trimmed.strip_prefix('[')
        && let Some(bracket_end) = rest.find(']')
    {
        let key = &rest[..bracket_end];
        if !key.is_empty() {
            let display = rest[bracket_end + 1..].trim_start();
            return (key.to_uppercase(), display.to_string());
        }
    }

    // Pattern: K) Label
    if trimmed.len() >= 2 {
        let bytes = trimmed.as_bytes();
        if bytes.get(1) == Some(&b')') {
            let display = trimmed[2..].trim_start();
            return (trimmed[..1].to_uppercase(), display.to_string());
        }
    }

    // Pattern: K - Label
    if trimmed.len() >= 4 && trimmed.as_bytes().get(1..4) == Some(b" - ".as_slice()) {
        let display = trimmed[4..].trim_start();
        return (trimmed[..1].to_uppercase(), display.to_string());
    }

    // Fallback: first character as key, full label as display
    let key = trimmed
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_default();
    (key, trimmed.to_string())
}

#[async_trait]
impl Handler for WaitForHumanHandler {
    #[allow(clippy::too_many_lines)]
    async fn execute(
        &self,
        node: &Node,
        context: &Context,
        graph: &Graph,
    ) -> AttractorResult<Outcome> {
        // 1. Derive choices from outgoing edges
        let edges = graph.outgoing_edges(&node.id);
        let choices: Vec<Choice> = edges
            .iter()
            .map(|edge| {
                let raw = if edge.label().is_empty() {
                    edge.to.clone()
                } else {
                    edge.label().to_string()
                };
                let (key, label) = parse_accelerator_label(&raw);
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
                description: None,
            })
            .collect();

        let text = node.get_str_attr("label").unwrap_or("Select an option:");
        let mut question = Question::multiple_choice(text, options);

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

        // 3. Build an Interview and conduct it
        let mut interview = Interview::single(question.clone(), &node.id);
        interview.stage_index = context.get_i64("internal.stage_index");

        // 3a. Resume recovery: if we have a DB connection, check for a
        // pending interview from a previous run at this node/stage_index.
        // If found, reuse its ID so external systems that already have
        // the question can still submit answers.
        if let Some(ref db_conn) = self.db_conn {
            match stencila_interviews::find_pending_interview(
                db_conn,
                &self.context_type,
                &self.context_id,
                &node.id,
                interview.stage_index,
            ) {
                Ok(Some((recovered_id, recovered_qids))) => {
                    tracing::debug!(
                        interview_id = %recovered_id,
                        node_id = %node.id,
                        "recovered pending interview from previous run"
                    );
                    interview.id = recovered_id;
                    // Propagate recovered question IDs so PersistentInterviewer's
                    // answer updates target the existing DB rows.
                    for (q, qid) in interview.questions.iter_mut().zip(recovered_qids) {
                        q.id = Some(qid);
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(
                        node_id = %node.id,
                        error = %e,
                        "failed to query for pending interview during resume; \
                         proceeding with a new interview ID"
                    );
                }
            }
        }

        // Use the interview's question (not the local `question`) because
        // resume recovery above may have set `question.id` on the interview copy.
        self.emitter.emit(PipelineEvent::InterviewQuestionAsked {
            interview_id: interview.id.clone(),
            node_id: node.id.clone(),
            question: interview.questions[0].clone(),
        });

        self.interviewer
            .conduct(&mut interview)
            .await
            .map_err(|e| match e {
                InterviewError::ChannelClosed => crate::error::AttractorError::HandlerFailed {
                    node_id: node.id.clone(),
                    reason: "interview channel closed".into(),
                },
                InterviewError::BackendFailure(msg) => {
                    crate::error::AttractorError::HandlerFailed {
                        node_id: node.id.clone(),
                        reason: format!("interview backend failure: {msg}"),
                    }
                }
                InterviewError::Cancelled => crate::error::AttractorError::HandlerFailed {
                    node_id: node.id.clone(),
                    reason: "interview cancelled".into(),
                },
            })?;

        let answer = interview.answers.first().cloned().ok_or_else(|| {
            crate::error::AttractorError::HandlerFailed {
                node_id: node.id.clone(),
                reason: "no answer after interview".into(),
            }
        })?;

        // 4. Handle timeout/skip
        if answer.is_timeout() {
            self.emitter.emit(PipelineEvent::InterviewTimedOut {
                interview_id: interview.id.clone(),
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
            interview_id: interview.id.clone(),
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
// explicitly — see `parse_accelerator_label` which is public for testing.
