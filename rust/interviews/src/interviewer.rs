//! Human-in-the-loop interviewer abstraction (§6.1–6.3).
//!
//! All human interaction goes through the [`Interviewer`] trait, which
//! supports asking questions, batching multiple questions, and sending
//! one-way informational messages.

use std::fmt;

use async_trait::async_trait;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during an interview interaction.
///
/// These represent infrastructure failures — the interviewer *could not
/// complete* the interaction. This is distinct from [`AnswerValue::Timeout`]
/// and [`AnswerValue::Skipped`], which are valid answer semantics (the
/// interaction completed, but the human didn't provide a substantive answer).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InterviewError {
    /// The answer channel was dropped (frontend disconnected).
    #[error("interview channel closed")]
    ChannelClosed,

    /// A backend (DB, network, or other infrastructure) failure occurred.
    #[error("interview backend failure: {0}")]
    BackendFailure(String),

    /// The interview was explicitly cancelled (e.g., pipeline abort).
    #[error("interview cancelled")]
    Cancelled,
}

/// The type of question being asked (§6.2).
// TODO(spec-ambiguity): §6.2 defines YES_NO/MULTIPLE_CHOICE/FREEFORM/CONFIRMATION
// but §11.8 uses SINGLE_SELECT/MULTI_SELECT/FREE_TEXT/CONFIRM. Using §6.2 names
// (normative section). (spec: §6.2 vs §11.8)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QuestionType {
    /// A yes/no binary choice.
    YesNo,
    /// Select one from a list of options.
    MultipleChoice,
    /// Select multiple options from a list (§11.8 `MULTI_SELECT`).
    MultiSelect,
    /// Free-form text input.
    Freeform,
    /// A yes/no confirmation (semantically distinct from `YesNo`).
    Confirmation,
}

impl fmt::Display for QuestionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::YesNo => f.write_str("YES_NO"),
            Self::MultipleChoice => f.write_str("MULTIPLE_CHOICE"),
            Self::MultiSelect => f.write_str("MULTI_SELECT"),
            Self::Freeform => f.write_str("FREEFORM"),
            Self::Confirmation => f.write_str("CONFIRMATION"),
        }
    }
}

/// A selectable option for multiple-choice and multi-select questions (§6.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestionOption {
    /// Accelerator key (e.g., `"Y"`, `"A"`).
    pub key: String,
    /// Display text (e.g., `"Yes, deploy to production"`).
    pub label: String,
    /// Explanatory text displayed alongside the label (e.g., `"Brief overview"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A question presented to a human (§6.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    /// Unique identifier for this question instance.
    ///
    /// Usually set by [`PersistentInterviewer`] to a per-question UUID
    /// (`question_id`) before delegating to the inner interviewer, so
    /// frontends and external systems can correlate submitted answers with
    /// `interview_questions.question_id` rows. Currently `None` unless set
    /// manually.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The question text to present.
    pub text: String,
    /// Short label displayed above the question text (e.g., `"Format"`, `"Sections"`).
    ///
    /// Distinct from `stage` (the originating pipeline node) and `text`
    /// (the full question). Used by frontends to render grouped/headed
    /// question forms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
    /// The question type, determining valid answers.
    pub question_type: QuestionType,
    /// Options for `MultipleChoice` and `MultiSelect` questions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<QuestionOption>,
    /// Default answer to use on timeout or skip, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Answer>,
    /// Maximum time (in seconds) to wait for a response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<f64>,
    /// Arbitrary key-value metadata for remote interviewers.
    ///
    /// Useful for passing extra context (pipeline name, run ID, urgency
    /// level, etc.) to web, email, or Slack frontends.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub metadata: IndexMap<String, serde_json::Value>,
}

impl Question {
    /// Create a yes/no question.
    #[must_use]
    pub fn yes_no(text: impl Into<String>) -> Self {
        Self {
            id: None,
            text: text.into(),
            header: None,
            question_type: QuestionType::YesNo,
            options: Vec::new(),
            default: None,
            timeout_seconds: None,
            metadata: IndexMap::new(),
        }
    }

    /// Create a confirmation question.
    #[must_use]
    pub fn confirmation(text: impl Into<String>) -> Self {
        Self {
            id: None,
            text: text.into(),
            header: None,
            question_type: QuestionType::Confirmation,
            options: Vec::new(),
            default: None,
            timeout_seconds: None,
            metadata: IndexMap::new(),
        }
    }

    /// Create a multiple-choice question.
    #[must_use]
    pub fn multiple_choice(text: impl Into<String>, options: Vec<QuestionOption>) -> Self {
        Self {
            id: None,
            text: text.into(),
            header: None,
            question_type: QuestionType::MultipleChoice,
            options,
            default: None,
            timeout_seconds: None,
            metadata: IndexMap::new(),
        }
    }

    /// Create a multi-select question (select multiple options from a list).
    #[must_use]
    pub fn multi_select(text: impl Into<String>, options: Vec<QuestionOption>) -> Self {
        Self {
            id: None,
            text: text.into(),
            header: None,
            question_type: QuestionType::MultiSelect,
            options,
            default: None,
            timeout_seconds: None,
            metadata: IndexMap::new(),
        }
    }

    /// Create a free-form text question.
    #[must_use]
    pub fn freeform(text: impl Into<String>) -> Self {
        Self {
            id: None,
            text: text.into(),
            header: None,
            question_type: QuestionType::Freeform,
            options: Vec::new(),
            default: None,
            timeout_seconds: None,
            metadata: IndexMap::new(),
        }
    }
}

/// The semantic value of an answer (§6.3).
///
/// Uses adjacently tagged serde representation (`"type"` + `"value"`) for
/// consistency with the codebase's enum tagging convention. Unit variants
/// (e.g., `Yes`) serialize as `{"type":"YES"}`, data variants (e.g.,
/// `Selected`) as `{"type":"SELECTED","value":"..."}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnswerValue {
    /// Affirmative response.
    Yes,
    /// Negative response.
    No,
    /// Human skipped the question.
    Skipped,
    /// No response within timeout.
    Timeout,
    /// A selected option key (for single-select multiple choice).
    Selected(String),
    /// Multiple selected option keys (for multi-select).
    MultiSelected(Vec<String>),
    /// Free-form text response.
    Text(String),
}

impl fmt::Display for AnswerValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Yes => f.write_str("YES"),
            Self::No => f.write_str("NO"),
            Self::Skipped => f.write_str("SKIPPED"),
            Self::Timeout => f.write_str("TIMEOUT"),
            Self::Selected(key) => write!(f, "SELECTED({key})"),
            Self::MultiSelected(keys) => write!(f, "MULTI_SELECTED({})", keys.join(",")),
            Self::Text(text) => write!(f, "TEXT({text})"),
        }
    }
}

/// An answer to a question (§6.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Answer {
    /// The answer value.
    pub value: AnswerValue,
    /// The full selected option, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_option: Option<QuestionOption>,
}

impl Answer {
    /// Create an answer with the given value and no selected option.
    #[must_use]
    pub fn new(value: AnswerValue) -> Self {
        Self {
            value,
            selected_option: None,
        }
    }

    /// Create an answer with a selected option.
    #[must_use]
    pub fn with_option(value: AnswerValue, option: QuestionOption) -> Self {
        Self {
            value,
            selected_option: Some(option),
        }
    }

    /// Check if this answer is a timeout.
    #[must_use]
    pub fn is_timeout(&self) -> bool {
        self.value == AnswerValue::Timeout
    }

    /// Check if this answer is skipped.
    #[must_use]
    pub fn is_skipped(&self) -> bool {
        self.value == AnswerValue::Skipped
    }
}

/// Produce a canonical string representation of an answer value.
///
/// This is the single source of truth for how answer values are
/// represented as strings, used by:
/// - `show_if` / `finish_if` condition evaluation
/// - context storage in pipeline handlers
/// - answer formatting in agent tools
///
/// For [`AnswerValue::Selected`], the option **label** is returned
/// (resolved via the question's options), falling back to the raw key
/// if no matching option is found. This means condition authors write
/// human-readable values (e.g., `show_if: "target == Production"`)
/// rather than internal keys.
#[must_use]
pub fn canonical_answer_string(value: &AnswerValue, question: &Question) -> String {
    match value {
        AnswerValue::Yes => "yes".to_string(),
        AnswerValue::No => "no".to_string(),
        AnswerValue::Skipped => "skipped".to_string(),
        AnswerValue::Timeout => "timeout".to_string(),
        AnswerValue::Text(text) => text.clone(),
        AnswerValue::Selected(key) => question
            .options
            .iter()
            .find(|o| o.key == *key)
            .map(|o| o.label.clone())
            .unwrap_or_else(|| key.clone()),
        AnswerValue::MultiSelected(keys) => keys
            .iter()
            .map(|key| {
                question
                    .options
                    .iter()
                    .find(|o| o.key == *key)
                    .map(|o| o.label.clone())
                    .unwrap_or_else(|| key.clone())
            })
            .collect::<Vec<_>>()
            .join(", "),
    }
}

/// An artifact attached to an interview for human review.
///
/// Attachments are lightweight references — they carry metadata (filename,
/// media type, size) and an optional URL, but never embed file bytes. For
/// local interviewers (TUI, CLI), the URL may use a `file://` scheme or be
/// `None` (the file is accessed directly). For remote interviewers (webhook,
/// email, Slack), the server/cloud layer populates `url` with an HTTP
/// download link before posting the envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attachment {
    /// Unique identifier for this attachment.
    pub id: String,
    /// Display filename (e.g., `"report-draft.docx"`).
    pub filename: String,
    /// MIME type (e.g., `"image/png"`, `"application/pdf"`).
    pub media_type: String,
    /// Download URL. Set by the server/cloud layer when the
    /// attachment is served over HTTP; `None` for local-only
    /// attachments (TUI, CLI) where the file is accessed directly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Size in bytes, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    /// Human-readable description (e.g., `"Draft report v2"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A group of questions presented to a human as a single interaction.
///
/// Single-question interviews are the common case (pipeline gates via
/// `WaitForHumanHandler`). Multi-question interviews are used by the
/// `ask_user` agent tool and by frontends like email/Slack that prefer
/// batched interactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interview {
    /// Unique identifier for this interview instance.
    pub id: String,

    /// Originating stage name (e.g., pipeline node ID, `"ask_user"`).
    pub stage: String,

    /// Introductory text displayed before the questions.
    ///
    /// Provides framing context — e.g., "Here's the quarterly report draft.
    /// Please review and answer the following questions."
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preamble: Option<String>,

    /// Artifacts attached for the human to review alongside the questions.
    ///
    /// Attachments are interview-level: a single attachment (e.g., a docx
    /// draft) often provides context for multiple questions. Individual
    /// questions can reference relevant attachment IDs via their `metadata`
    /// (e.g., `"attachment_ids": ["att-1"]`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,

    /// The questions to present (one or more).
    pub questions: Vec<Question>,

    /// Answers received (parallel to `questions`; empty until answered).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub answers: Vec<Answer>,

    /// Pipeline stage index, used to disambiguate multiple visits to
    /// the same node during loops/retries. Set by the engine before
    /// calling `conduct()`. `None` for non-pipeline contexts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_index: Option<i64>,

    /// Interview-level metadata (pipeline name, urgency, etc.)
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub metadata: IndexMap<String, serde_json::Value>,
}

impl Interview {
    /// Create a single-question interview with a generated UUID v7 ID.
    #[must_use]
    pub fn single(question: Question, stage: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::now_v7().to_string(),
            questions: vec![question],
            answers: Vec::new(),
            stage: stage.into(),
            preamble: None,
            attachments: Vec::new(),
            stage_index: None,
            metadata: IndexMap::new(),
        }
    }

    /// Create a multi-question interview with a generated UUID v7 ID.
    #[must_use]
    pub fn batch(questions: Vec<Question>, stage: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::now_v7().to_string(),
            questions,
            answers: Vec::new(),
            stage: stage.into(),
            preamble: None,
            attachments: Vec::new(),
            stage_index: None,
            metadata: IndexMap::new(),
        }
    }

    /// Set the preamble text for this interview.
    #[must_use]
    pub fn with_preamble(mut self, preamble: impl Into<String>) -> Self {
        self.preamble = Some(preamble.into());
        self
    }

    /// Set the attachments for this interview.
    #[must_use]
    pub fn with_attachments(mut self, attachments: Vec<Attachment>) -> Self {
        self.attachments = attachments;
        self
    }

    /// Append a single attachment to this interview.
    #[must_use]
    pub fn with_attachment(mut self, attachment: Attachment) -> Self {
        self.attachments.push(attachment);
        self
    }
}

/// Parse a raw text answer into a typed [`Answer`] based on the question type.
///
/// This is useful for any text-based frontend (TUI, CLI, web form) that
/// receives user input as a string and needs to convert it to a typed answer.
///
/// For [`QuestionType::YesNo`] and [`QuestionType::Confirmation`], recognizes
/// `y`, `yes`, `true`, `1` as [`AnswerValue::Yes`] and `n`, `no`, `false`, `0`
/// as [`AnswerValue::No`] (case-insensitive). Unrecognized input falls back to
/// [`AnswerValue::Text`].
///
/// For [`QuestionType::MultipleChoice`], matches against option keys first,
/// then labels (case-insensitive). Unmatched input falls back to
/// [`AnswerValue::Text`].
///
/// For [`QuestionType::Freeform`], always returns [`AnswerValue::Text`].
#[must_use]
pub fn parse_answer_text(text: &str, question: &Question) -> Answer {
    let trimmed = text.trim();
    match question.question_type {
        QuestionType::YesNo | QuestionType::Confirmation => {
            let lower = trimmed.to_ascii_lowercase();
            if matches!(lower.as_str(), "y" | "yes" | "true" | "1") {
                Answer::new(AnswerValue::Yes)
            } else if matches!(lower.as_str(), "n" | "no" | "false" | "0") {
                Answer::new(AnswerValue::No)
            } else {
                Answer::new(AnswerValue::Text(trimmed.to_string()))
            }
        }
        QuestionType::MultipleChoice => {
            if let Some(opt) = question
                .options
                .iter()
                .find(|o| o.key.eq_ignore_ascii_case(trimmed))
            {
                Answer::with_option(AnswerValue::Selected(opt.key.clone()), opt.clone())
            } else if let Some(opt) = question
                .options
                .iter()
                .find(|o| o.label.eq_ignore_ascii_case(trimmed))
            {
                Answer::with_option(AnswerValue::Selected(opt.key.clone()), opt.clone())
            } else {
                Answer::new(AnswerValue::Text(trimmed.to_string()))
            }
        }
        QuestionType::MultiSelect => {
            let parts: Vec<&str> = trimmed
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            let mut selected_keys = Vec::new();
            for part in &parts {
                if let Some(opt) = question.options.iter().find(|o| {
                    o.key.eq_ignore_ascii_case(part) || o.label.eq_ignore_ascii_case(part)
                }) && !selected_keys.contains(&opt.key)
                {
                    selected_keys.push(opt.key.clone());
                }
            }
            if selected_keys.is_empty() {
                Answer::new(AnswerValue::Text(trimmed.to_string()))
            } else {
                Answer::new(AnswerValue::MultiSelected(selected_keys))
            }
        }
        QuestionType::Freeform => Answer::new(AnswerValue::Text(trimmed.to_string())),
    }
}

/// Trait for human interaction frontends (§6.1).
///
/// Implementations provide different ways of presenting questions
/// to humans: CLI, web UI, pre-filled queues for testing, etc.
#[async_trait]
pub trait Interviewer: Send + Sync {
    /// Ask a single question and wait for an answer.
    ///
    /// Returns `Ok(answer)` on success, or `Err(InterviewError)` if the
    /// interviewer could not complete the interaction (channel closed,
    /// backend failure, cancelled). Note that timeouts and skips are
    /// *not* errors — they are valid [`AnswerValue`] variants.
    async fn ask(&self, question: &Question) -> Result<Answer, InterviewError>;

    /// Conduct an interview: present one or more questions and collect answers.
    ///
    /// This is the primary method for multi-question interviews. Frontends
    /// that support batch presentation (web forms, email, Slack) override
    /// this to render all questions together. The default calls `ask()`
    /// sequentially.
    async fn conduct(&self, interview: &mut Interview) -> Result<(), InterviewError> {
        interview.answers.clear();
        for q in &interview.questions {
            interview.answers.push(self.ask(q).await?);
        }
        Ok(())
    }

    /// Send a one-way informational message (no response expected).
    fn inform(&self, _message: &str, _stage: &str) {
        // Default: no-op
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn freeform_question() -> Question {
        Question::freeform("What is your name?")
    }

    fn yes_no_question() -> Question {
        Question::yes_no("Do you agree?")
    }

    fn multiple_choice_question() -> Question {
        Question::multiple_choice(
            "Pick one:",
            vec![
                QuestionOption {
                    key: "A".to_string(),
                    label: "Option Alpha".to_string(),
                    description: None,
                },
                QuestionOption {
                    key: "B".to_string(),
                    label: "Option Beta".to_string(),
                    description: None,
                },
            ],
        )
    }

    fn multi_select_question() -> Question {
        Question::multi_select(
            "Select all that apply:",
            vec![
                QuestionOption {
                    key: "X".to_string(),
                    label: "Option X".to_string(),
                    description: Some("First option".to_string()),
                },
                QuestionOption {
                    key: "Y".to_string(),
                    label: "Option Y".to_string(),
                    description: None,
                },
                QuestionOption {
                    key: "Z".to_string(),
                    label: "Option Z".to_string(),
                    description: Some("Third option".to_string()),
                },
            ],
        )
    }

    #[test]
    fn parse_freeform_answer() {
        let q = freeform_question();
        let answer = parse_answer_text("hello world", &q);
        assert_eq!(answer.value, AnswerValue::Text("hello world".to_string()));
    }

    #[test]
    fn parse_yes_no_yes() {
        let q = yes_no_question();
        assert_eq!(parse_answer_text("y", &q).value, AnswerValue::Yes);
        assert_eq!(parse_answer_text("YES", &q).value, AnswerValue::Yes);
        assert_eq!(parse_answer_text("true", &q).value, AnswerValue::Yes);
    }

    #[test]
    fn parse_yes_no_no() {
        let q = yes_no_question();
        assert_eq!(parse_answer_text("n", &q).value, AnswerValue::No);
        assert_eq!(parse_answer_text("NO", &q).value, AnswerValue::No);
        assert_eq!(parse_answer_text("false", &q).value, AnswerValue::No);
    }

    #[test]
    fn parse_multiple_choice_by_key() {
        let q = multiple_choice_question();
        let answer = parse_answer_text("A", &q);
        assert_eq!(answer.value, AnswerValue::Selected("A".to_string()));
        assert!(answer.selected_option.is_some());
    }

    #[test]
    fn parse_multiple_choice_by_label() {
        let q = multiple_choice_question();
        let answer = parse_answer_text("option beta", &q);
        assert_eq!(answer.value, AnswerValue::Selected("B".to_string()));
    }

    #[test]
    fn parse_multiple_choice_no_match() {
        let q = multiple_choice_question();
        let answer = parse_answer_text("unknown", &q);
        assert_eq!(answer.value, AnswerValue::Text("unknown".to_string()));
    }

    #[test]
    fn parse_multi_select_by_keys() {
        let q = multi_select_question();
        let answer = parse_answer_text("X, Z", &q);
        assert_eq!(
            answer.value,
            AnswerValue::MultiSelected(vec!["X".to_string(), "Z".to_string()])
        );
    }

    #[test]
    fn parse_multi_select_by_labels() {
        let q = multi_select_question();
        let answer = parse_answer_text("option x, option z", &q);
        assert_eq!(
            answer.value,
            AnswerValue::MultiSelected(vec!["X".to_string(), "Z".to_string()])
        );
    }

    #[test]
    fn parse_multi_select_single_item() {
        let q = multi_select_question();
        let answer = parse_answer_text("Y", &q);
        assert_eq!(
            answer.value,
            AnswerValue::MultiSelected(vec!["Y".to_string()])
        );
    }

    #[test]
    fn parse_multi_select_no_match() {
        let q = multi_select_question();
        let answer = parse_answer_text("unknown", &q);
        assert_eq!(answer.value, AnswerValue::Text("unknown".to_string()));
    }

    #[test]
    fn parse_multi_select_deduplicates() {
        let q = multi_select_question();
        let answer = parse_answer_text("X, X, option x", &q);
        assert_eq!(
            answer.value,
            AnswerValue::MultiSelected(vec!["X".to_string()])
        );
    }

    #[test]
    fn question_type_display_multi_select() {
        assert_eq!(QuestionType::MultiSelect.to_string(), "MULTI_SELECT");
    }

    #[test]
    fn answer_value_display_multi_selected() {
        let val = AnswerValue::MultiSelected(vec!["A".into(), "B".into()]);
        assert_eq!(val.to_string(), "MULTI_SELECTED(A,B)");
    }

    #[test]
    fn question_serde_roundtrip() -> serde_json::Result<()> {
        let mut q = Question::freeform("What?");
        q.id = Some("q-123".to_string());
        q.header = Some("Header".to_string());
        q.metadata
            .insert("urgency".to_string(), serde_json::json!("high"));

        let json = serde_json::to_string(&q)?;
        let q2: Question = serde_json::from_str(&json)?;
        assert_eq!(q2.id, Some("q-123".to_string()));
        assert_eq!(q2.header, Some("Header".to_string()));
        assert_eq!(q2.text, "What?");
        assert_eq!(q2.question_type, QuestionType::Freeform);
        assert_eq!(q2.metadata["urgency"], serde_json::json!("high"));
        Ok(())
    }

    #[test]
    fn question_type_serde_roundtrip() -> serde_json::Result<()> {
        let qt = QuestionType::MultiSelect;
        let json = serde_json::to_string(&qt)?;
        assert_eq!(json, "\"MULTI_SELECT\"");
        let qt2: QuestionType = serde_json::from_str(&json)?;
        assert_eq!(qt2, QuestionType::MultiSelect);
        Ok(())
    }

    #[test]
    fn answer_serde_roundtrip() -> serde_json::Result<()> {
        let answer = Answer::new(AnswerValue::MultiSelected(vec!["A".into(), "B".into()]));
        let json = serde_json::to_string(&answer)?;
        let answer2: Answer = serde_json::from_str(&json)?;
        assert_eq!(answer2, answer);
        Ok(())
    }

    #[test]
    fn answer_value_adjacently_tagged_format() -> serde_json::Result<()> {
        // Unit variants: {"type":"YES"} — no "value" key
        let json = serde_json::to_string(&AnswerValue::Yes)?;
        assert_eq!(json, r#"{"type":"YES"}"#);

        // Data variants: {"type":"SELECTED","value":"A"}
        let json = serde_json::to_string(&AnswerValue::Selected("A".into()))?;
        assert_eq!(json, r#"{"type":"SELECTED","value":"A"}"#);

        // Vec data: {"type":"MULTI_SELECTED","value":["A","B"]}
        let json =
            serde_json::to_string(&AnswerValue::MultiSelected(vec!["A".into(), "B".into()]))?;
        assert_eq!(json, r#"{"type":"MULTI_SELECTED","value":["A","B"]}"#);
        Ok(())
    }

    #[test]
    fn question_option_with_description_serde() -> serde_json::Result<()> {
        let opt = QuestionOption {
            key: "A".to_string(),
            label: "Alpha".to_string(),
            description: Some("The first letter".to_string()),
        };
        let json = serde_json::to_string(&opt)?;
        assert!(json.contains("description"));
        let opt2: QuestionOption = serde_json::from_str(&json)?;
        assert_eq!(opt2.description, Some("The first letter".to_string()));
        Ok(())
    }

    #[test]
    fn question_option_without_description_serde() -> serde_json::Result<()> {
        let opt = QuestionOption {
            key: "B".to_string(),
            label: "Beta".to_string(),
            description: None,
        };
        let json = serde_json::to_string(&opt)?;
        assert!(!json.contains("description"));
        let opt2: QuestionOption = serde_json::from_str(&json)?;
        assert_eq!(opt2.description, None);
        Ok(())
    }

    #[test]
    fn interview_single_creates_uuid() {
        let q = Question::yes_no("Proceed?");
        let interview = Interview::single(q, "gate-1");
        assert!(!interview.id.is_empty());
        assert_eq!(interview.questions.len(), 1);
        assert!(interview.answers.is_empty());
        assert_eq!(interview.stage, "gate-1");
        assert!(interview.metadata.is_empty());
    }

    #[test]
    fn interview_batch_creates_uuid() {
        let qs = vec![Question::yes_no("Q1?"), Question::freeform("Q2?")];
        let interview = Interview::batch(qs, "ask_user");
        assert!(!interview.id.is_empty());
        assert_eq!(interview.questions.len(), 2);
        assert!(interview.answers.is_empty());
        assert_eq!(interview.stage, "ask_user");
    }

    #[test]
    fn interview_single_sets_stage_explicitly() {
        let q = Question::freeform("Name?");
        let interview = Interview::single(q, "my-stage");
        assert_eq!(interview.stage, "my-stage");
    }

    #[test]
    fn interview_serde_roundtrip() -> serde_json::Result<()> {
        let mut interview = Interview::single(Question::yes_no("Continue?"), "gate");
        interview.answers.push(Answer::new(AnswerValue::Yes));
        interview
            .metadata
            .insert("urgency".to_string(), serde_json::json!("high"));

        let json = serde_json::to_string(&interview)?;
        let interview2: Interview = serde_json::from_str(&json)?;
        assert_eq!(interview2.id, interview.id);
        assert_eq!(interview2.questions.len(), 1);
        assert_eq!(interview2.answers.len(), 1);
        assert_eq!(interview2.answers[0].value, AnswerValue::Yes);
        assert_eq!(interview2.stage, "gate");
        assert_eq!(interview2.metadata["urgency"], serde_json::json!("high"));
        Ok(())
    }

    #[test]
    fn interview_serde_empty_answers_omitted() -> serde_json::Result<()> {
        let interview = Interview::single(Question::yes_no("OK?"), "s");
        let json = serde_json::to_string(&interview)?;
        assert!(!json.contains("answers"));
        Ok(())
    }

    #[test]
    fn interview_serde_empty_metadata_omitted() -> serde_json::Result<()> {
        let interview = Interview::single(Question::yes_no("OK?"), "s");
        let json = serde_json::to_string(&interview)?;
        assert!(!json.contains("metadata"));
        Ok(())
    }

    #[test]
    fn interview_single_has_no_preamble_or_attachments() {
        let interview = Interview::single(Question::yes_no("OK?"), "s");
        assert!(interview.preamble.is_none());
        assert!(interview.attachments.is_empty());
    }

    #[test]
    fn interview_batch_has_no_preamble_or_attachments() {
        let qs = vec![Question::yes_no("Q1?")];
        let interview = Interview::batch(qs, "s");
        assert!(interview.preamble.is_none());
        assert!(interview.attachments.is_empty());
    }

    #[test]
    fn interview_with_preamble_builder() {
        let interview = Interview::single(Question::yes_no("OK?"), "s")
            .with_preamble("Please review the attached draft.");
        assert_eq!(
            interview.preamble.as_deref(),
            Some("Please review the attached draft.")
        );
    }

    #[test]
    fn interview_with_attachments_builder() {
        let att = Attachment {
            id: "att-1".into(),
            filename: "report.pdf".into(),
            media_type: "application/pdf".into(),
            url: None,
            size_bytes: Some(1024),
            description: Some("Q3 report".into()),
        };
        let interview =
            Interview::single(Question::yes_no("OK?"), "s").with_attachments(vec![att.clone()]);
        assert_eq!(interview.attachments.len(), 1);
        assert_eq!(interview.attachments[0], att);
    }

    #[test]
    fn interview_with_attachment_builder_appends() {
        let att1 = Attachment {
            id: "att-1".into(),
            filename: "a.pdf".into(),
            media_type: "application/pdf".into(),
            url: None,
            size_bytes: None,
            description: None,
        };
        let att2 = Attachment {
            id: "att-2".into(),
            filename: "b.png".into(),
            media_type: "image/png".into(),
            url: Some("https://example.com/b.png".into()),
            size_bytes: Some(2048),
            description: Some("Screenshot".into()),
        };
        let interview = Interview::single(Question::yes_no("OK?"), "s")
            .with_attachment(att1)
            .with_attachment(att2);
        assert_eq!(interview.attachments.len(), 2);
        assert_eq!(interview.attachments[0].id, "att-1");
        assert_eq!(interview.attachments[1].id, "att-2");
    }

    #[test]
    fn interview_serde_empty_preamble_omitted() -> serde_json::Result<()> {
        let interview = Interview::single(Question::yes_no("OK?"), "s");
        let json = serde_json::to_string(&interview)?;
        assert!(!json.contains("preamble"));
        Ok(())
    }

    #[test]
    fn interview_serde_empty_attachments_omitted() -> serde_json::Result<()> {
        let interview = Interview::single(Question::yes_no("OK?"), "s");
        let json = serde_json::to_string(&interview)?;
        assert!(!json.contains("attachments"));
        Ok(())
    }

    #[test]
    fn interview_serde_roundtrip_with_preamble_and_attachments() -> serde_json::Result<()> {
        let att = Attachment {
            id: "att-1".into(),
            filename: "draft.docx".into(),
            media_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                .into(),
            url: Some("https://cdn.example.com/draft.docx".into()),
            size_bytes: Some(51200),
            description: Some("Draft report v2".into()),
        };
        let interview = Interview::single(Question::yes_no("OK?"), "gate")
            .with_preamble("Review the attached draft")
            .with_attachment(att.clone());

        let json = serde_json::to_string(&interview)?;
        assert!(json.contains("preamble"));
        assert!(json.contains("attachments"));

        let interview2: Interview = serde_json::from_str(&json)?;
        assert_eq!(
            interview2.preamble.as_deref(),
            Some("Review the attached draft")
        );
        assert_eq!(interview2.attachments.len(), 1);
        assert_eq!(interview2.attachments[0], att);
        Ok(())
    }

    #[test]
    fn attachment_serde_roundtrip() -> serde_json::Result<()> {
        let att = Attachment {
            id: "att-1".into(),
            filename: "report.pdf".into(),
            media_type: "application/pdf".into(),
            url: Some("https://example.com/report.pdf".into()),
            size_bytes: Some(1024),
            description: Some("Quarterly report".into()),
        };
        let json = serde_json::to_string(&att)?;
        let att2: Attachment = serde_json::from_str(&json)?;
        assert_eq!(att, att2);
        Ok(())
    }

    #[test]
    fn attachment_serde_optional_fields_omitted() -> serde_json::Result<()> {
        let att = Attachment {
            id: "att-1".into(),
            filename: "file.txt".into(),
            media_type: "text/plain".into(),
            url: None,
            size_bytes: None,
            description: None,
        };
        let json = serde_json::to_string(&att)?;
        assert!(!json.contains("url"));
        assert!(!json.contains("size_bytes"));
        assert!(!json.contains("description"));

        let att2: Attachment = serde_json::from_str(&json)?;
        assert_eq!(att, att2);
        Ok(())
    }
}
