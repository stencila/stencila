//! Human-in-the-loop interviewer abstraction (§6.1–6.3).
//!
//! All human interaction goes through the [`Interviewer`] trait, which
//! supports asking questions, batching multiple questions, and sending
//! one-way informational messages.

use std::fmt;

/// The type of question being asked (§6.2).
// TODO(spec-ambiguity): §6.2 defines YES_NO/MULTIPLE_CHOICE/FREEFORM/CONFIRMATION
// but §11.8 uses SINGLE_SELECT/MULTI_SELECT/FREE_TEXT/CONFIRM. Using §6.2 names
// (normative section). (spec: §6.2 vs §11.8)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuestionType {
    /// A yes/no binary choice.
    YesNo,
    /// Select one from a list of options.
    MultipleChoice,
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
            Self::Freeform => f.write_str("FREEFORM"),
            Self::Confirmation => f.write_str("CONFIRMATION"),
        }
    }
}

/// A selectable option for multiple-choice questions (§6.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionOption {
    /// Accelerator key (e.g., `"Y"`, `"A"`).
    pub key: String,
    /// Display text (e.g., `"Yes, deploy to production"`).
    pub label: String,
}

/// A question presented to a human (§6.2).
#[derive(Debug, Clone)]
pub struct Question {
    /// The question text to present.
    pub text: String,
    /// The question type, determining valid answers.
    pub question_type: QuestionType,
    /// Options for `MultipleChoice` questions.
    pub options: Vec<QuestionOption>,
    /// Default answer to use on timeout or skip, if any.
    pub default: Option<Answer>,
    /// Maximum time (in seconds) to wait for a response.
    pub timeout_seconds: Option<f64>,
    /// Originating stage name (for display).
    pub stage: String,
}

impl Question {
    /// Create a yes/no question.
    #[must_use]
    pub fn yes_no(text: impl Into<String>, stage: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            question_type: QuestionType::YesNo,
            options: Vec::new(),
            default: None,
            timeout_seconds: None,
            stage: stage.into(),
        }
    }

    /// Create a confirmation question.
    #[must_use]
    pub fn confirmation(text: impl Into<String>, stage: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            question_type: QuestionType::Confirmation,
            options: Vec::new(),
            default: None,
            timeout_seconds: None,
            stage: stage.into(),
        }
    }

    /// Create a multiple-choice question.
    #[must_use]
    pub fn multiple_choice(
        text: impl Into<String>,
        options: Vec<QuestionOption>,
        stage: impl Into<String>,
    ) -> Self {
        Self {
            text: text.into(),
            question_type: QuestionType::MultipleChoice,
            options,
            default: None,
            timeout_seconds: None,
            stage: stage.into(),
        }
    }

    /// Create a free-form text question.
    #[must_use]
    pub fn freeform(text: impl Into<String>, stage: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            question_type: QuestionType::Freeform,
            options: Vec::new(),
            default: None,
            timeout_seconds: None,
            stage: stage.into(),
        }
    }
}

/// The semantic value of an answer (§6.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnswerValue {
    /// Affirmative response.
    Yes,
    /// Negative response.
    No,
    /// Human skipped the question.
    Skipped,
    /// No response within timeout.
    Timeout,
    /// A selected option key (for multiple choice).
    Selected(String),
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
            Self::Text(text) => write!(f, "TEXT({text})"),
        }
    }
}

/// An answer to a question (§6.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    /// The answer value.
    pub value: AnswerValue,
    /// The full selected option, if applicable.
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

/// Trait for human interaction frontends (§6.1).
///
/// Implementations provide different ways of presenting questions
/// to humans: CLI, web UI, pre-filled queues for testing, etc.
pub trait Interviewer: Send + Sync {
    /// Ask a single question and wait for an answer.
    fn ask(&self, question: &Question) -> Answer;

    /// Ask multiple questions and return answers in order.
    ///
    /// Default implementation calls [`ask`](Self::ask) for each question.
    fn ask_multiple(&self, questions: &[Question]) -> Vec<Answer> {
        questions.iter().map(|q| self.ask(q)).collect()
    }

    /// Send a one-way informational message (no response expected).
    fn inform(&self, _message: &str, _stage: &str) {
        // Default: no-op
    }
}
