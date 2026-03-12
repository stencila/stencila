//! Wait-for-human handler (§4.6).
//!
//! Blocks pipeline execution until a human selects an option derived
//! from the node's outgoing edges. Implements the human-in-the-loop
//! pattern using the [`Interviewer`] trait.
//!
//! ## Extended question types
//!
//! Nodes can specify `question-type` to override the default
//! multiple-choice behavior:
//!
//! - `question-type="freeform"` — presents a free-form text prompt
//! - `question-type="yes-no"` — presents a yes/no question
//! - `question-type="confirm"` — presents a confirmation prompt
//!
//! When omitted, the handler derives a multiple-choice question from
//! outgoing edge labels (the original behavior).
//!
//! ## Storing answers in context
//!
//! The `store` attribute writes the human's answer into a named
//! context key so that later workflow nodes can reference it via
//! `$KEY` in their prompts:
//!
//! ```dot
//! Feedback [ask="What should change?", question-type="freeform", store="human.feedback"]
//! ```

use std::sync::Arc;

use async_trait::async_trait;
use indexmap::IndexMap;

use stencila_interviews::conduct::conduct_conditional;
use stencila_interviews::spec::InterviewSpec;

use crate::context::Context;
use crate::error::AttractorResult;
use crate::events::{EventEmitter, NoOpEmitter, PipelineEvent};
use crate::graph::{Graph, Node};
use crate::handler::Handler;
use crate::interviewer::{
    Answer, AnswerValue, Interview, InterviewError, Interviewer, Question, QuestionOption,
    QuestionType,
};
use crate::types::Outcome;

/// Handler for `wait.human` nodes that presents questions to a human.
///
/// By default, choices are derived from the node's outgoing edges and
/// presented as a multiple-choice question. The `question-type`
/// attribute overrides this to support freeform text, yes/no, and
/// confirmation prompts. The `store` attribute writes the answer into
/// the pipeline context so later nodes can interpolate it.
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

impl WaitForHumanHandler {
    /// Execute a multi-question interview defined by an `InterviewSpec`.
    ///
    /// Parses the spec from YAML (with JSON fallback), conducts the
    /// interview, stores per-question answers in context, and routes
    /// based on the first `single-select` question's answer.
    #[allow(clippy::too_many_lines)]
    async fn execute_interview_spec(
        &self,
        node: &Node,
        context: &Context,
        graph: &Graph,
        spec_str: &str,
    ) -> AttractorResult<Outcome> {
        let store_key = node.get_str_attr("store").map(ToString::to_string);

        // 1. Parse the spec from YAML (fall back to JSON)
        let spec = InterviewSpec::parse(spec_str).map_err(|reason| {
            crate::error::AttractorError::HandlerFailed {
                node_id: node.id.clone(),
                reason,
            }
        })?;

        // 2. Validate semantic correctness (show-if, finish-if, duplicates)
        if let Err(errors) = spec.validate() {
            return Err(crate::error::AttractorError::HandlerFailed {
                node_id: node.id.clone(),
                reason: format!("invalid interview spec: {}", errors.join("; ")),
            });
        }

        // 3. Conditional specs are conducted progressively (one question
        //    at a time) so that show-if / finish-if can be evaluated.
        //    Resume recovery is not supported for conditional interviews.
        if spec.is_conditional() {
            return self
                .execute_conditional_interview_spec(node, graph, &spec, store_key.as_deref())
                .await;
        }

        // 4. Convert spec to Interview (non-conditional batch path)
        let mut interview = spec.to_interview(&node.id).map_err(|reason| {
            crate::error::AttractorError::HandlerFailed {
                node_id: node.id.clone(),
                reason,
            }
        })?;
        interview.stage_index = context.get_i64("internal.stage_index");

        // 5. Resume recovery
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

        // 6. Emit events for each question
        for question in &interview.questions {
            self.emitter.emit(PipelineEvent::InterviewQuestionAsked {
                interview_id: interview.id.clone(),
                node_id: node.id.clone(),
                question: question.clone(),
            });
        }

        // 7. Conduct the interview
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

        // 8. Store answers and find routing answer.
        //    For non-conditional specs, spec index == interview index.
        let mut context_updates = IndexMap::new();
        let mut routing_answer: Option<Answer> = None;
        let mut routing_question_options: Vec<QuestionOption> = Vec::new();

        for (i, question) in interview.questions.iter().enumerate() {
            let answer = interview.answers.get(i).cloned().ok_or_else(|| {
                crate::error::AttractorError::HandlerFailed {
                    node_id: node.id.clone(),
                    reason: format!("no answer for question {i} after interview"),
                }
            })?;

            // Store the canonical (human-readable) value so that stored
            // values match what show-if / finish-if conditions compare.
            let canonical =
                stencila_interviews::interviewer::canonical_answer_string(&answer.value, question);

            // Store under per-question `store` key
            if let Some(ref store) = spec.questions.get(i).and_then(|q| q.store.clone()) {
                context_updates.insert(store.clone(), serde_json::Value::String(canonical.clone()));
            }

            // Also store under node-level `store` for the first answer (compatibility)
            if i == 0
                && let Some(ref key) = store_key
            {
                context_updates.insert(key.clone(), serde_json::Value::String(canonical));
            }

            // Track the first single-select question's answer for routing
            if routing_answer.is_none() && question.r#type == QuestionType::SingleSelect {
                routing_answer = Some(answer);
                routing_question_options.clone_from(&question.options);
            }
        }

        self.emitter.emit(PipelineEvent::InterviewAnswerReceived {
            interview_id: interview.id.clone(),
            node_id: node.id.clone(),
        });

        // 9. Routing: derive choices from outgoing edges
        let choices = choices_from_edges(graph, &node.id);

        if let Some(routing_ans) = routing_answer {
            // Routing requires at least one outgoing edge
            if choices.is_empty() {
                return Ok(Outcome::fail(
                    "interview has a routing question but no outgoing edges",
                ));
            }

            // Handle timeout/skip for routing answer
            if routing_ans.is_timeout() {
                self.emitter.emit(PipelineEvent::InterviewTimedOut {
                    interview_id: interview.id.clone(),
                    node_id: node.id.clone(),
                });

                if let Some(default_target) = node.get_str_attr("human.default_choice")
                    && let Some(choice) = find_choice_by_str(default_target, &choices)
                {
                    let mut outcome = build_human_outcome(choice);
                    outcome.context_updates.extend(context_updates);
                    return Ok(outcome);
                }
                return Ok(Outcome::retry("human gate timeout, no default"));
            }

            if routing_ans.is_skipped() {
                return Ok(Outcome::fail("human skipped interaction"));
            }

            // Choice-based routing using first single-select answer
            let Some(selected) =
                find_matching_choice(&routing_ans, &choices, &routing_question_options)
            else {
                return Ok(Outcome::fail("answer did not match any available choice"));
            };

            let mut outcome = build_human_outcome(selected);
            outcome.context_updates.extend(context_updates);
            Ok(outcome)
        } else if choices.is_empty() {
            // Terminal interview node — no routing question, no outgoing
            // edges. Succeed with just the stored context updates.
            Ok(Outcome {
                context_updates,
                ..Outcome::success()
            })
        } else {
            // No routing question — follow first outgoing edge
            Ok(Outcome {
                status: crate::types::StageStatus::Success,
                suggested_next_ids: vec![choices[0].target.clone()],
                context_updates,
                ..Outcome::success()
            })
        }
    }

    /// Execute a conditional interview spec progressively.
    ///
    /// Uses [`conduct_conditional`] which asks questions one at a time,
    /// evaluating `show-if` / `finish-if` between questions. The returned
    /// [`ConductedInterview`] maps each asked question back to its spec
    /// index, so store keys are resolved correctly even when questions are
    /// skipped.
    ///
    /// Resume recovery is not supported for conditional interviews — if
    /// the process crashes mid-interview, it restarts from the first
    /// question.
    #[allow(clippy::too_many_lines)]
    async fn execute_conditional_interview_spec(
        &self,
        node: &Node,
        graph: &Graph,
        spec: &InterviewSpec,
        store_key: Option<&str>,
    ) -> AttractorResult<Outcome> {
        let result = conduct_conditional(spec, self.interviewer.as_ref(), &node.id)
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

        let interview = &result.interview;

        // Emit per-question events for parity with the batch path
        for question in &interview.questions {
            self.emitter.emit(PipelineEvent::InterviewQuestionAsked {
                interview_id: interview.id.clone(),
                node_id: node.id.clone(),
                question: question.clone(),
            });
        }

        // Store answers using spec_indices for correct mapping
        let mut context_updates = IndexMap::new();
        let mut routing_answer: Option<Answer> = None;
        let mut routing_question_options: Vec<QuestionOption> = Vec::new();

        for (i, question) in interview.questions.iter().enumerate() {
            let answer = interview.answers.get(i).cloned().ok_or_else(|| {
                crate::error::AttractorError::HandlerFailed {
                    node_id: node.id.clone(),
                    reason: format!("no answer for question {i} after conditional interview"),
                }
            })?;

            // Use spec_indices to find the original spec question and
            // store the canonical (human-readable) value so that stored
            // values match what show-if / finish-if conditions compare.
            let spec_idx = result.spec_indices[i];
            if let Some(ref store) = spec.questions.get(spec_idx).and_then(|q| q.store.clone()) {
                context_updates.insert(
                    store.clone(),
                    serde_json::Value::String(
                        stencila_interviews::interviewer::canonical_answer_string(
                            &answer.value,
                            question,
                        ),
                    ),
                );
            }

            if i == 0
                && let Some(key) = store_key
            {
                context_updates.insert(
                    key.to_string(),
                    serde_json::Value::String(
                        stencila_interviews::interviewer::canonical_answer_string(
                            &answer.value,
                            question,
                        ),
                    ),
                );
            }

            if routing_answer.is_none() && question.r#type == QuestionType::SingleSelect {
                routing_answer = Some(answer);
                routing_question_options.clone_from(&question.options);
            }
        }

        self.emitter.emit(PipelineEvent::InterviewAnswerReceived {
            interview_id: interview.id.clone(),
            node_id: node.id.clone(),
        });

        // Routing mirrors the non-conditional path, including timeout
        // and default-choice handling.
        let choices = choices_from_edges(graph, &node.id);

        if let Some(routing_ans) = routing_answer {
            if choices.is_empty() {
                return Ok(Outcome::fail(
                    "interview has a routing question but no outgoing edges",
                ));
            }

            if routing_ans.is_timeout() {
                self.emitter.emit(PipelineEvent::InterviewTimedOut {
                    interview_id: interview.id.clone(),
                    node_id: node.id.clone(),
                });

                if let Some(default_target) = node.get_str_attr("human.default_choice")
                    && let Some(choice) = find_choice_by_str(default_target, &choices)
                {
                    let mut outcome = build_human_outcome(choice);
                    outcome.context_updates.extend(context_updates);
                    return Ok(outcome);
                }
                return Ok(Outcome::retry("human gate timeout, no default"));
            }

            if routing_ans.is_skipped() {
                return Ok(Outcome::fail("human skipped interaction"));
            }

            let Some(selected) =
                find_matching_choice(&routing_ans, &choices, &routing_question_options)
            else {
                return Ok(Outcome::fail("answer did not match any available choice"));
            };

            let mut outcome = build_human_outcome(selected);
            outcome.context_updates.extend(context_updates);
            Ok(outcome)
        } else if choices.is_empty() {
            Ok(Outcome {
                context_updates,
                ..Outcome::success()
            })
        } else {
            Ok(Outcome {
                status: crate::types::StageStatus::Success,
                suggested_next_ids: vec![choices[0].target.clone()],
                context_updates,
                ..Outcome::success()
            })
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

/// Derive choices from a node's outgoing edges.
fn choices_from_edges(graph: &Graph, node_id: &str) -> Vec<Choice> {
    graph
        .outgoing_edges(node_id)
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
        .collect()
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

/// Determine the question type from a node's `question_type` attribute.
///
/// Returns `None` when the attribute is absent (caller should use the
/// default multiple-choice-from-edges behavior).
fn parse_question_type(node: &Node) -> Option<QuestionType> {
    node.get_str_attr("question_type").and_then(|s| match s {
        "freeform" => Some(QuestionType::Freeform),
        "yes_no" | "yes-no" => Some(QuestionType::YesNo),
        "confirm" => Some(QuestionType::Confirm),
        "single_select" | "single-select" | "multi_choice" | "multi-choice" | "multiple_choice"
        | "multiple-choice" => Some(QuestionType::SingleSelect),
        "multi_select" | "multi-select" | "multiple_select" | "multiple-select" => {
            Some(QuestionType::MultiSelect)
        }
        _ => None,
    })
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
        // Multi-question interview path: if the node has an `interview`
        // attribute, parse and execute the full interview spec.
        if let Some(interview_spec_str) = node.get_str_attr("interview") {
            return self
                .execute_interview_spec(node, context, graph, interview_spec_str)
                .await;
        }

        let store_key = node.get_str_attr("store").map(ToString::to_string);
        let question_type = parse_question_type(node);

        // 1. Derive choices from outgoing edges (used for routing and
        //    for multiple-choice question building)
        let choices = choices_from_edges(graph, &node.id);

        // For multiple-choice (the default), we need at least one edge.
        // For other question types, we need at least one edge for routing
        // but don't derive question options from them.
        if choices.is_empty() {
            return Ok(Outcome::fail("No outgoing edges for human gate"));
        }

        // 2. Build question based on question_type
        let text = node.get_str_attr("label").unwrap_or("Select an option:");
        let is_choice_based = question_type.is_none()
            || matches!(
                question_type,
                Some(QuestionType::SingleSelect | QuestionType::MultiSelect)
            );

        let mut question = match question_type {
            Some(QuestionType::Freeform) => Question::freeform(text),
            Some(QuestionType::YesNo) => Question::yes_no(text),
            Some(QuestionType::Confirm) => Question::confirm(text),
            Some(QuestionType::MultiSelect) => {
                let options: Vec<QuestionOption> = choices
                    .iter()
                    .map(|c| QuestionOption {
                        key: c.key.clone(),
                        label: c.label.clone(),
                        description: None,
                    })
                    .collect();
                Question::multi_select(text, options)
            }
            // Default: MultipleChoice (original behavior)
            None | Some(QuestionType::SingleSelect) => {
                let options: Vec<QuestionOption> = choices
                    .iter()
                    .map(|c| QuestionOption {
                        key: c.key.clone(),
                        label: c.label.clone(),
                        description: None,
                    })
                    .collect();
                Question::single_select(text, options)
            }
        };

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

            if is_choice_based
                && let Some(default_target) = node.get_str_attr("human.default_choice")
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

        // 5. Build outcome based on question type
        if is_choice_based {
            // Choice-based: find matching choice for routing
            let Some(selected) = find_matching_choice(&answer, &choices, &question.options) else {
                return Ok(Outcome::fail("answer did not match any available choice"));
            };

            let mut outcome = build_human_outcome(selected);

            // For choice-based questions with a store key, store the
            // canonical (human-readable) value for consistency with
            // condition evaluation.
            if let Some(ref key) = store_key {
                outcome.context_updates.insert(
                    key.clone(),
                    serde_json::Value::String(
                        stencila_interviews::interviewer::canonical_answer_string(
                            &answer.value,
                            &question,
                        ),
                    ),
                );
            }

            Ok(outcome)
        } else {
            // Non-choice (freeform, yes_no, confirm): follow the
            // first outgoing edge unconditionally.
            let mut updates = IndexMap::new();
            if let Some(ref key) = store_key {
                updates.insert(
                    key.clone(),
                    serde_json::Value::String(
                        stencila_interviews::interviewer::canonical_answer_string(
                            &answer.value,
                            &question,
                        ),
                    ),
                );
            }

            Ok(Outcome {
                status: crate::types::StageStatus::Success,
                suggested_next_ids: vec![choices[0].target.clone()],
                context_updates: updates,
                ..Outcome::success()
            })
        }
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
/// - `Selected(key)` → case-insensitive key match against edge-derived
///   choice keys, then label-based fallback: resolve the selected key to
///   its option label (via `answer.selected_option` or `question_options`)
///   and match that label against edge choice labels
/// - `Text(text)` → case-insensitive match by key, then by label
/// - `Yes` → first choice (convenience for single-choice gates)
/// - All other variants → no match (returns `None`)
///
/// The label-based fallback for `Selected` answers handles the case where
/// interview-spec option keys (auto-assigned A, B, C…) differ from
/// edge-derived keys (first letter of each label). The spec pseudocode
/// matches only by key; we extend with label matching to improve UX when
/// humans type full labels.
fn find_matching_choice<'a>(
    answer: &Answer,
    choices: &'a [Choice],
    question_options: &[QuestionOption],
) -> Option<&'a Choice> {
    match &answer.value {
        AnswerValue::Selected(key) => {
            // Try matching the selected key directly against edge-derived keys.
            choices
                .iter()
                .find(|c| c.key.eq_ignore_ascii_case(key))
                .or_else(|| {
                    // When the question comes from an interview spec, its
                    // auto-assigned option keys (A, B, C…) may differ from
                    // the edge-derived keys (first letter of each label).
                    // Resolve the selected key to the option label, then
                    // match that label against edge labels.
                    let label = answer
                        .selected_option
                        .as_ref()
                        .map(|o| o.label.as_str())
                        .or_else(|| {
                            question_options
                                .iter()
                                .find(|o| o.key.eq_ignore_ascii_case(key))
                                .map(|o| o.label.as_str())
                        })?;
                    choices.iter().find(|c| c.label.eq_ignore_ascii_case(label))
                })
        }
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
