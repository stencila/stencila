use std::collections::HashSet;

use stencila_attractor::interviewer::{
    Answer, AnswerValue, Interview, InterviewError, Interviewer, Question, QuestionType,
    interview_helpers, parse_answer_text,
};

use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};

// ─── Interview source and status ────────────────────────────────────

/// Source context for an interview.
#[derive(Debug, Clone)]
pub enum InterviewSource {
    /// Interview from an agent session's `ask_user` tool.
    Agent,
    /// Interview from a workflow's `wait.human` gate.
    Workflow,
}

fn join_keys(keys: &[&str]) -> String {
    match keys {
        [] => String::new(),
        [one] => (*one).to_string(),
        [first, second] => format!("{first} or {second}"),
        _ => {
            let mut joined = keys[..keys.len() - 1].join(", ");
            joined.push_str(", or ");
            joined.push_str(keys[keys.len() - 1]);
            joined
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PreviewSelection {
    pub yes_no: Option<bool>,
    pub selected: Option<usize>,
}

pub fn is_answered_draft(draft: &DraftAnswer) -> bool {
    match draft {
        DraftAnswer::Pending => false,
        DraftAnswer::Text(text) => !text.trim().is_empty(),
        DraftAnswer::YesNo(value) => value.is_some(),
        DraftAnswer::Selected(value) => value.is_some(),
        DraftAnswer::MultiSelected(values) => !values.is_empty(),
    }
}

pub fn preview_selection(input: &str, question: &Question) -> PreviewSelection {
    let trimmed = input.trim();
    match question.r#type {
        QuestionType::YesNo | QuestionType::Confirm => {
            let lower = trimmed.to_ascii_lowercase();
            let yes = ["y", "ye", "yes", "t", "tr", "tru", "true", "1"];
            let no = ["n", "no", "f", "fa", "fal", "fals", "false", "0"];
            PreviewSelection {
                yes_no: if yes.iter().any(|candidate| candidate == &lower) {
                    Some(true)
                } else if no.iter().any(|candidate| candidate == &lower) {
                    Some(false)
                } else {
                    None
                },
                selected: None,
            }
        }
        QuestionType::SingleSelect => PreviewSelection {
            yes_no: None,
            selected: find_option_by_key_or_label(trimmed, &question.options),
        },
        _ => PreviewSelection::default(),
    }
}

pub fn preview_multi_select(input: &str, question: &Question) -> HashSet<usize> {
    input
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .filter_map(|part| find_option_by_key_or_label(part, &question.options))
        .collect()
}

/// Find the index of an option by key (exact) or label (prefix), with key
/// matches taking priority.
///
/// When the user types a letter that is both a valid option key and a prefix
/// of an earlier option's label, the key match must win. Without two-pass
/// resolution, `position` returns whichever option matched first, which may
/// be the wrong one when the label prefix happens to appear before the key.
fn find_option_by_key_or_label(
    input: &str,
    options: &[stencila_attractor::interviewer::QuestionOption],
) -> Option<usize> {
    let lower = input.to_ascii_lowercase();
    options
        .iter()
        .position(|o| o.key.eq_ignore_ascii_case(&lower))
        .or_else(|| {
            options
                .iter()
                .position(|o| o.label.to_ascii_lowercase().starts_with(&lower))
        })
}

/// Find the index of an option by exact key or exact label match
/// (case-insensitive). Used for answer submission where prefix matching
/// is not appropriate.
fn find_option_index_exact(
    input: &str,
    options: &[stencila_attractor::interviewer::QuestionOption],
) -> Option<usize> {
    options
        .iter()
        .position(|o| o.key.eq_ignore_ascii_case(input))
        .or_else(|| {
            options
                .iter()
                .position(|o| o.label.eq_ignore_ascii_case(input))
        })
}

/// Format option keys for use in validation error messages.
fn option_keys_display(question: &Question) -> String {
    question
        .options
        .iter()
        .map(|o| o.key.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Parse comma-separated input into a set of option indices.
///
/// Each part is looked up via exact key/label matching. Returns `Err` with
/// a user-facing message if any part is empty or unrecognised.
fn parse_multi_select_indices(input: &str, question: &Question) -> Result<HashSet<usize>, String> {
    let choices_err = || {
        format!(
            "invalid: choose one or more: {}",
            option_keys_display(question),
        )
    };
    let parts: Vec<&str> = input
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if parts.is_empty() {
        return Err(choices_err());
    }
    let mut indices = HashSet::new();
    for part in &parts {
        if let Some(idx) = find_option_index_exact(part, &question.options) {
            indices.insert(idx);
        } else {
            return Err(choices_err());
        }
    }
    Ok(indices)
}

/// Determine the default option focus for a question that has no
/// draft-driven focus. Returns `Some(0)` for yes/no, confirm, or
/// questions with options; `None` for freeform questions with no options.
fn default_option_focus(question: &Question) -> Option<usize> {
    if matches!(question.r#type, QuestionType::YesNo | QuestionType::Confirm)
        || !question.options.is_empty()
    {
        Some(0)
    } else {
        None
    }
}

/// Status of an interview in the transcript.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterviewStatus {
    /// Interview is active and awaiting answers.
    Active,
    /// All answers have been submitted.
    Completed,
    /// Interview was cancelled by the user.
    Cancelled,
}

// ─── Draft answers ──────────────────────────────────────────────────

/// A draft answer being composed by the user.
#[derive(Debug, Clone)]
pub enum DraftAnswer {
    /// Not yet answered.
    Pending,
    /// Freeform text input.
    Text(String),
    /// Yes/No or Confirmation selection.
    YesNo(Option<bool>),
    /// Single selection from options (index into question.options).
    Selected(Option<usize>),
    /// Multiple selections from options (indices into question.options).
    MultiSelected(HashSet<usize>),
}

impl DraftAnswer {
    /// Create the appropriate initial draft for a question type.
    ///
    /// When the question has a default answer, the draft is pre-populated
    /// so the user can press Enter to accept it.
    pub fn for_question(question: &Question) -> Self {
        if let Some(default) = &question.default {
            match (&question.r#type, &default.value) {
                (QuestionType::YesNo | QuestionType::Confirm, AnswerValue::Yes) => {
                    return Self::YesNo(Some(true));
                }
                (QuestionType::YesNo | QuestionType::Confirm, AnswerValue::No) => {
                    return Self::YesNo(Some(false));
                }
                (QuestionType::SingleSelect, AnswerValue::Selected(key)) => {
                    if let Some(idx) = question
                        .options
                        .iter()
                        .position(|o| o.key.eq_ignore_ascii_case(key))
                    {
                        return Self::Selected(Some(idx));
                    }
                }
                (QuestionType::MultiSelect, AnswerValue::MultiSelected(keys)) => {
                    let indices: HashSet<usize> = keys
                        .iter()
                        .filter_map(|key| {
                            question
                                .options
                                .iter()
                                .position(|o| o.key.eq_ignore_ascii_case(key))
                        })
                        .collect();
                    if !indices.is_empty() {
                        return Self::MultiSelected(indices);
                    }
                }
                (QuestionType::Freeform, AnswerValue::Text(text)) => {
                    return Self::Text(text.clone());
                }
                _ => {}
            }
        }

        match question.r#type {
            QuestionType::Freeform => Self::Pending,
            QuestionType::YesNo | QuestionType::Confirm => Self::YesNo(None),
            QuestionType::SingleSelect => Self::Selected(None),
            QuestionType::MultiSelect => Self::MultiSelected(HashSet::new()),
        }
    }

    /// Convert the draft answer to text suitable for restoring to the input area.
    pub fn to_input_text(&self, question: &Question) -> String {
        match self {
            Self::Pending | Self::YesNo(None) | Self::Selected(None) => String::new(),
            Self::Text(s) => s.clone(),
            Self::YesNo(Some(true)) => "y".to_string(),
            Self::YesNo(Some(false)) => "n".to_string(),
            Self::Selected(Some(idx)) => question
                .options
                .get(*idx)
                .map_or(String::new(), |o| o.key.clone()),
            Self::MultiSelected(indices) => {
                let mut keys: Vec<&str> = indices
                    .iter()
                    .filter_map(|i| question.options.get(*i).map(|o| o.key.as_str()))
                    .collect();
                keys.sort_unstable();
                keys.join(", ")
            }
        }
    }

    /// Convert to a finalized `Answer` using the question context.
    pub fn to_answer(&self, question: &Question) -> Answer {
        match self {
            Self::Pending | Self::YesNo(None) | Self::Selected(None) => {
                Answer::new(AnswerValue::Skipped)
            }
            Self::Text(s) => parse_answer_text(s, question),
            Self::YesNo(Some(true)) => Answer::new(AnswerValue::Yes),
            Self::YesNo(Some(false)) => Answer::new(AnswerValue::No),
            Self::Selected(Some(idx)) => {
                if let Some(opt) = question.options.get(*idx) {
                    Answer::with_option(AnswerValue::Selected(opt.key.clone()), opt.clone())
                } else {
                    Answer::new(AnswerValue::Skipped)
                }
            }
            Self::MultiSelected(indices) => {
                let keys: Vec<String> = indices
                    .iter()
                    .filter_map(|i| question.options.get(*i).map(|o| o.key.clone()))
                    .collect();
                if keys.is_empty() {
                    Answer::new(AnswerValue::Skipped)
                } else {
                    Answer::new(AnswerValue::MultiSelected(keys))
                }
            }
        }
    }
}

// ─── Interview result ───────────────────────────────────────────────

/// Result sent back through the answer channel.
pub enum InterviewResult {
    /// All answers completed.
    Completed(Vec<Answer>),
    /// Interview was cancelled by the user.
    Cancelled,
}

// ─── Transient interview state ──────────────────────────────────────

/// Transient UI state for an in-progress interview.
///
/// Stores only the mutable interaction cursor and draft answers.
/// The interview content itself lives in `AppMessage::Interview`.
pub struct InterviewState {
    /// Interview ID, matching the `AppMessage::Interview` in the message list.
    pub interview_id: String,

    /// Message index of the corresponding `AppMessage::Interview`.
    pub msg_index: usize,

    /// Which question is currently focused (0-indexed).
    pub current_question: usize,

    /// Which option is currently focused for each question (0-indexed).
    pub option_focus: Vec<Option<usize>>,

    /// Draft answers for each question (parallel to interview.questions).
    pub draft_answers: Vec<DraftAnswer>,

    /// Channel to send completed answers back to the interviewer.
    pub answer_tx: Option<oneshot::Sender<InterviewResult>>,

    /// Transient validation error message for the hints line.
    pub validation_error: Option<String>,
}

impl InterviewState {
    /// Create a new interview state for the given interview.
    pub fn new(
        interview: &Interview,
        msg_index: usize,
        answer_tx: oneshot::Sender<InterviewResult>,
    ) -> Self {
        let draft_answers = interview
            .questions
            .iter()
            .map(DraftAnswer::for_question)
            .collect();
        let option_focus = interview
            .questions
            .iter()
            .map(|question| {
                if matches!(question.r#type, QuestionType::YesNo | QuestionType::Confirm)
                    || !question.options.is_empty()
                {
                    Some(0)
                } else {
                    None
                }
            })
            .collect();

        Self {
            interview_id: interview.id.clone(),
            msg_index,
            current_question: 0,
            option_focus,
            draft_answers,
            answer_tx: Some(answer_tx),
            validation_error: None,
        }
    }

    /// Get the currently focused option for the current question.
    pub fn current_option_focus(&self) -> Option<usize> {
        self.option_focus
            .get(self.current_question)
            .copied()
            .flatten()
    }

    /// Set the focused option for the current question.
    pub fn set_current_option_focus(&mut self, focus: Option<usize>) {
        if let Some(current) = self.option_focus.get_mut(self.current_question) {
            *current = focus;
        }
    }

    /// Sync option focus for the current question based on its draft answer.
    pub fn sync_focus_from_draft(&mut self, question: &Question) {
        let focus = match self.draft_answers.get(self.current_question) {
            Some(DraftAnswer::YesNo(Some(true))) => Some(0),
            Some(DraftAnswer::YesNo(Some(false))) => Some(1),
            Some(DraftAnswer::Selected(Some(idx))) => Some(*idx),
            Some(DraftAnswer::MultiSelected(indices)) => indices
                .iter()
                .min()
                .copied()
                .or(default_option_focus(question)),
            _ => default_option_focus(question),
        };
        self.set_current_option_focus(focus);
    }

    /// Move option focus within the current question by one step.
    pub fn move_option_focus(&mut self, question: &Question, delta: isize) -> bool {
        let len = if matches!(question.r#type, QuestionType::YesNo | QuestionType::Confirm) {
            2
        } else {
            question.options.len()
        };
        if len == 0 {
            return false;
        }

        let current = self.current_option_focus().unwrap_or(0);
        let next = match delta.cmp(&0) {
            std::cmp::Ordering::Less => current.saturating_sub(1),
            std::cmp::Ordering::Greater => (current + 1).min(len.saturating_sub(1)),
            std::cmp::Ordering::Equal => current,
        };

        self.set_current_option_focus(Some(next));
        self.validation_error = None;
        true
    }

    /// Toggle or select the currently focused option for the current question.
    pub fn activate_focused_option(&mut self, question: &Question) -> bool {
        let Some(focus) = self.current_option_focus() else {
            return false;
        };

        match question.r#type {
            QuestionType::YesNo | QuestionType::Confirm => {
                self.draft_answers[self.current_question] = DraftAnswer::YesNo(Some(focus == 0));
                self.validation_error = None;
                true
            }
            QuestionType::SingleSelect => {
                self.draft_answers[self.current_question] = DraftAnswer::Selected(Some(focus));
                self.validation_error = None;
                true
            }
            QuestionType::MultiSelect => {
                if let DraftAnswer::MultiSelected(indices) =
                    &mut self.draft_answers[self.current_question]
                {
                    if !indices.insert(focus) {
                        indices.remove(&focus);
                    }
                    self.validation_error = None;
                    true
                } else {
                    false
                }
            }
            QuestionType::Freeform => false,
        }
    }

    /// Try to set the answer for the current question from input text.
    ///
    /// Returns `true` if the input was valid for this question type,
    /// `false` if validation failed (with `validation_error` set).
    #[allow(clippy::too_many_lines)]
    pub fn try_set_answer_from_input(&mut self, input: &str, question: &Question) -> bool {
        let trimmed = input.trim();

        // When the input is empty and the draft already holds a value
        // (pre-populated from a default or prior selection), accept it.
        if trimmed.is_empty() && is_answered_draft(&self.draft_answers[self.current_question]) {
            self.validation_error = None;
            return true;
        }

        // For single-selection questions (yes/no, confirm, multiple
        // choice) with empty input, accept the currently focused option so
        // the user can press Enter without typing. Multi-select is excluded
        // because the user should explicitly toggle items before submitting.
        if trimmed.is_empty()
            && matches!(
                question.r#type,
                QuestionType::YesNo | QuestionType::Confirm | QuestionType::SingleSelect
            )
            && self.activate_focused_option(question)
        {
            self.validation_error = None;
            return true;
        }

        match question.r#type {
            QuestionType::Freeform => {
                if trimmed.is_empty() {
                    self.validation_error = Some("invalid: enter your answer".to_string());
                    return false;
                }
                self.draft_answers[self.current_question] = DraftAnswer::Text(trimmed.to_string());
                self.validation_error = None;
                true
            }
            QuestionType::YesNo | QuestionType::Confirm => {
                let lower = trimmed.to_ascii_lowercase();
                if matches!(lower.as_str(), "y" | "yes" | "true" | "1") {
                    self.draft_answers[self.current_question] = DraftAnswer::YesNo(Some(true));
                    self.validation_error = None;
                    true
                } else if matches!(lower.as_str(), "n" | "no" | "false" | "0") {
                    self.draft_answers[self.current_question] = DraftAnswer::YesNo(Some(false));
                    self.validation_error = None;
                    true
                } else {
                    self.validation_error = Some("invalid: answer yes or no".to_string());
                    false
                }
            }
            QuestionType::SingleSelect => {
                if let Some(idx) = find_option_index_exact(trimmed, &question.options) {
                    self.draft_answers[self.current_question] = DraftAnswer::Selected(Some(idx));
                    self.validation_error = None;
                    true
                } else {
                    let keys: Vec<&str> = question.options.iter().map(|o| o.key.as_str()).collect();
                    self.validation_error = Some(format!("invalid: choose {}", join_keys(&keys)));
                    false
                }
            }
            QuestionType::MultiSelect => match parse_multi_select_indices(trimmed, question) {
                Ok(indices) => {
                    self.draft_answers[self.current_question] = DraftAnswer::MultiSelected(indices);
                    self.validation_error = None;
                    true
                }
                Err(msg) => {
                    self.validation_error = Some(msg);
                    false
                }
            },
        }
    }

    /// Evaluate whether a question should be shown based on its `show_if`
    /// condition and the answers collected so far.
    pub fn should_show_question(&self, q_idx: usize, questions: &[Question]) -> bool {
        if questions[q_idx].show_if.is_none() {
            return true;
        }

        // Build stored answers from draft answers up to q_idx.
        let draft_answers: Vec<_> = questions
            .iter()
            .zip(self.draft_answers.iter())
            .take(q_idx)
            .map(|(q, d)| d.to_answer(q))
            .collect();
        let stored = interview_helpers::build_stored_answers(questions, &draft_answers, q_idx);
        interview_helpers::should_show_question(&questions[q_idx], &stored, false)
    }

    /// Check whether a `finish_if` condition was triggered by the answer
    /// to the current question.
    fn is_finish_triggered(&self, questions: &[Question]) -> bool {
        let q = &questions[self.current_question];
        let answer = self.draft_answers[self.current_question].to_answer(q);
        interview_helpers::is_finish_triggered(q, &answer)
    }

    /// Advance to the next question, skipping questions whose `show_if`
    /// conditions are false. Returns `true` if all questions are done.
    pub fn advance_with_conditions(&mut self, questions: &[Question]) -> bool {
        // Check if finish_if is triggered on the current question.
        if self.is_finish_triggered(questions) {
            self.validation_error = None;
            self.current_question = self.draft_answers.len();
            return true;
        }

        loop {
            self.current_question += 1;
            if self.current_question >= self.draft_answers.len() {
                self.validation_error = None;
                return true;
            }
            if self.should_show_question(self.current_question, questions) {
                self.validation_error = None;
                return false;
            }
        }
    }

    /// Advance to the next question. Returns `true` if all questions are done.
    pub fn advance(&mut self) -> bool {
        self.current_question += 1;
        self.validation_error = None;
        self.current_question >= self.draft_answers.len()
    }

    /// Go back to the previous visible question, skipping questions whose
    /// `show_if` conditions are false. Returns `true` if successful.
    pub fn back_with_conditions(&mut self, questions: &[Question]) -> bool {
        loop {
            if self.current_question == 0 {
                return false;
            }
            self.current_question -= 1;
            if self.should_show_question(self.current_question, questions) {
                self.validation_error = None;
                return true;
            }
        }
    }

    /// Go back to the previous question, returning `true` if successful.
    pub fn back(&mut self) -> bool {
        if self.current_question == 0 {
            return false;
        }
        self.current_question -= 1;
        self.validation_error = None;
        true
    }

    /// Finalize all draft answers into `Answer` values.
    ///
    /// Questions that were skipped by `show_if` conditions produce
    /// [`AnswerValue::Skipped`].
    pub fn finalize_answers(&self, questions: &[Question]) -> Vec<Answer> {
        self.draft_answers
            .iter()
            .zip(questions)
            .enumerate()
            .map(|(i, (draft, q))| {
                if self.should_show_question(i, questions) {
                    draft.to_answer(q)
                } else {
                    Answer::new(AnswerValue::Skipped)
                }
            })
            .collect()
    }
}

// ─── Unified TUI Interviewer ────────────────────────────────────────

/// A pending interview delivered through the channel from the interviewer.
pub struct PendingTuiInterview {
    pub interview: Interview,
    pub result_tx: oneshot::Sender<InterviewResult>,
}

impl std::fmt::Debug for PendingTuiInterview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingTuiInterview")
            .field("interview_id", &self.interview.id)
            .finish_non_exhaustive()
    }
}

/// Unified TUI interviewer for both agent and workflow contexts.
///
/// Overrides `conduct()` to send the full `Interview` through the channel,
/// allowing the TUI to render all questions with their context.
pub struct TuiInterviewer {
    interview_tx: mpsc::UnboundedSender<PendingTuiInterview>,
}

impl TuiInterviewer {
    pub fn new(interview_tx: mpsc::UnboundedSender<PendingTuiInterview>) -> Self {
        Self { interview_tx }
    }
}

#[async_trait]
impl Interviewer for TuiInterviewer {
    async fn ask(&self, question: &Question) -> Result<Answer, InterviewError> {
        let mut interview = Interview::single(question.clone(), "ask");
        self.conduct(&mut interview).await?;
        interview
            .answers
            .into_iter()
            .next()
            .ok_or(InterviewError::Cancelled)
    }

    async fn conduct(&self, interview: &mut Interview) -> Result<(), InterviewError> {
        let (result_tx, result_rx) = oneshot::channel();
        self.interview_tx
            .send(PendingTuiInterview {
                interview: interview.clone(),
                result_tx,
            })
            .map_err(|_| InterviewError::ChannelClosed)?;

        match result_rx.await {
            Ok(InterviewResult::Completed(answers)) => {
                interview.answers = answers;
                Ok(())
            }
            Ok(InterviewResult::Cancelled) => Err(InterviewError::Cancelled),
            Err(_) => Err(InterviewError::ChannelClosed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stencila_attractor::interviewer::QuestionOption;

    #[test]
    fn draft_answer_for_question_types() {
        let q = Question::freeform("test");
        assert!(matches!(
            DraftAnswer::for_question(&q),
            DraftAnswer::Pending
        ));

        let q = Question::yes_no("test");
        assert!(matches!(
            DraftAnswer::for_question(&q),
            DraftAnswer::YesNo(None)
        ));

        let q = Question::confirm("test");
        assert!(matches!(
            DraftAnswer::for_question(&q),
            DraftAnswer::YesNo(None)
        ));

        let q = Question::single_select(
            "test",
            vec![QuestionOption {
                key: "a".to_string(),
                label: "Option A".to_string(),
                description: None,
            }],
        );
        assert!(matches!(
            DraftAnswer::for_question(&q),
            DraftAnswer::Selected(None)
        ));

        let q = Question::multi_select(
            "test",
            vec![QuestionOption {
                key: "a".to_string(),
                label: "Option A".to_string(),
                description: None,
            }],
        );
        assert!(matches!(
            DraftAnswer::for_question(&q),
            DraftAnswer::MultiSelected(_)
        ));
    }

    #[test]
    fn try_set_freeform_answer() {
        let q = Question::freeform("test");
        let interview = Interview::single(q.clone(), "test");
        let (tx, _rx) = oneshot::channel();
        let mut state = InterviewState::new(&interview, 0, tx);

        assert!(!state.try_set_answer_from_input("", &q));
        assert!(state.validation_error.is_some());

        assert!(state.try_set_answer_from_input("hello", &q));
        assert!(state.validation_error.is_none());
    }

    #[test]
    fn try_set_yes_no_answer() {
        let q = Question::yes_no("test");
        let interview = Interview::single(q.clone(), "test");
        let (tx, _rx) = oneshot::channel();
        let mut state = InterviewState::new(&interview, 0, tx);

        assert!(!state.try_set_answer_from_input("maybe", &q));
        assert!(state.validation_error.is_some());

        assert!(state.try_set_answer_from_input("y", &q));
        assert!(matches!(
            state.draft_answers[0],
            DraftAnswer::YesNo(Some(true))
        ));

        assert!(state.try_set_answer_from_input("n", &q));
        assert!(matches!(
            state.draft_answers[0],
            DraftAnswer::YesNo(Some(false))
        ));
    }

    #[test]
    fn try_set_multi_choice_answer() {
        let q = Question::single_select(
            "test",
            vec![
                QuestionOption {
                    key: "a".to_string(),
                    label: "Option A".to_string(),
                    description: None,
                },
                QuestionOption {
                    key: "b".to_string(),
                    label: "Option B".to_string(),
                    description: None,
                },
            ],
        );
        let interview = Interview::single(q.clone(), "test");
        let (tx, _rx) = oneshot::channel();
        let mut state = InterviewState::new(&interview, 0, tx);

        assert!(!state.try_set_answer_from_input("c", &q));
        assert!(state.validation_error.is_some());

        assert!(state.try_set_answer_from_input("a", &q));
        assert!(matches!(
            state.draft_answers[0],
            DraftAnswer::Selected(Some(0))
        ));

        assert!(state.try_set_answer_from_input("Option B", &q));
        assert!(matches!(
            state.draft_answers[0],
            DraftAnswer::Selected(Some(1))
        ));
    }

    #[test]
    fn try_set_multi_select_answer() {
        let q = Question::multi_select(
            "test",
            vec![
                QuestionOption {
                    key: "a".to_string(),
                    label: "Option A".to_string(),
                    description: None,
                },
                QuestionOption {
                    key: "b".to_string(),
                    label: "Option B".to_string(),
                    description: None,
                },
            ],
        );
        let interview = Interview::single(q.clone(), "test");
        let (tx, _rx) = oneshot::channel();
        let mut state = InterviewState::new(&interview, 0, tx);

        assert!(!state.try_set_answer_from_input("", &q));
        assert!(state.validation_error.is_some());

        assert!(state.try_set_answer_from_input("a, b", &q));
        if let DraftAnswer::MultiSelected(ref indices) = state.draft_answers[0] {
            assert!(indices.contains(&0));
            assert!(indices.contains(&1));
        } else {
            panic!("Expected MultiSelected");
        }
    }

    #[test]
    fn advance_and_back() {
        let questions = vec![
            Question::freeform("q1"),
            Question::freeform("q2"),
            Question::freeform("q3"),
        ];
        let interview = Interview::batch(questions, "test");
        let (tx, _rx) = oneshot::channel();
        let mut state = InterviewState::new(&interview, 0, tx);

        assert_eq!(state.current_question, 0);
        assert!(!state.advance());
        assert_eq!(state.current_question, 1);
        assert!(!state.advance());
        assert_eq!(state.current_question, 2);
        assert!(state.advance()); // all done

        state.current_question = 2;
        assert!(state.back());
        assert_eq!(state.current_question, 1);
        assert!(state.back());
        assert_eq!(state.current_question, 0);
        assert!(!state.back()); // can't go before 0
    }

    #[test]
    fn finalize_answers() {
        let questions = vec![Question::yes_no("test"), Question::freeform("name")];
        let interview = Interview::batch(questions.clone(), "test");
        let (tx, _rx) = oneshot::channel();
        let mut state = InterviewState::new(&interview, 0, tx);

        state.draft_answers[0] = DraftAnswer::YesNo(Some(true));
        state.draft_answers[1] = DraftAnswer::Text("Alice".to_string());

        let answers = state.finalize_answers(&questions);
        assert_eq!(answers.len(), 2);
        assert_eq!(answers[0].value, AnswerValue::Yes);
        assert_eq!(answers[1].value, AnswerValue::Text("Alice".to_string()));
    }

    #[tokio::test]
    async fn tui_interviewer_conduct_sends_and_receives() -> Result<(), Box<dyn std::error::Error>>
    {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let interviewer = TuiInterviewer::new(tx);
        let question = Question::freeform("What?");

        let ask_handle = tokio::spawn({
            let q = question.clone();
            async move { interviewer.ask(&q).await }
        });

        let pending = rx.recv().await.ok_or("expected pending interview")?;
        assert_eq!(pending.interview.questions.len(), 1);
        assert_eq!(pending.interview.questions[0].text, "What?");

        pending
            .result_tx
            .send(InterviewResult::Completed(vec![Answer::new(
                AnswerValue::Text("Answer".to_string()),
            )]))
            .map_err(|_| "failed to send result")?;

        let answer = ask_handle.await??;
        assert_eq!(answer.value, AnswerValue::Text("Answer".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn tui_interviewer_cancelled() -> Result<(), Box<dyn std::error::Error>> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let interviewer = TuiInterviewer::new(tx);
        let question = Question::freeform("What?");

        let ask_handle = tokio::spawn({
            let q = question.clone();
            async move { interviewer.ask(&q).await }
        });

        let pending = rx.recv().await.ok_or("expected pending interview")?;
        pending
            .result_tx
            .send(InterviewResult::Cancelled)
            .map_err(|_| "failed to send result")?;

        let result = ask_handle.await?;
        assert!(matches!(result, Err(InterviewError::Cancelled)));
        Ok(())
    }

    #[test]
    fn to_input_text_roundtrips() {
        let q = Question::single_select(
            "test",
            vec![
                QuestionOption {
                    key: "a".to_string(),
                    label: "Option A".to_string(),
                    description: None,
                },
                QuestionOption {
                    key: "b".to_string(),
                    label: "Option B".to_string(),
                    description: None,
                },
            ],
        );

        let draft = DraftAnswer::Selected(Some(1));
        assert_eq!(draft.to_input_text(&q), "b");

        let draft = DraftAnswer::YesNo(Some(true));
        assert_eq!(draft.to_input_text(&q), "y");

        let draft = DraftAnswer::Text("hello".to_string());
        assert_eq!(draft.to_input_text(&q), "hello");

        let draft = DraftAnswer::Pending;
        assert_eq!(draft.to_input_text(&q), "");
    }

    #[test]
    fn draft_answer_prepopulated_from_default_yes_no() {
        let mut q = Question::yes_no("proceed?");
        q.default = Some(Answer::new(AnswerValue::Yes));
        assert!(matches!(
            DraftAnswer::for_question(&q),
            DraftAnswer::YesNo(Some(true))
        ));

        q.default = Some(Answer::new(AnswerValue::No));
        assert!(matches!(
            DraftAnswer::for_question(&q),
            DraftAnswer::YesNo(Some(false))
        ));
    }

    #[test]
    fn draft_answer_prepopulated_from_default_multi_choice() {
        let mut q = Question::single_select(
            "pick",
            vec![
                QuestionOption {
                    key: "a".to_string(),
                    label: "Option A".to_string(),
                    description: None,
                },
                QuestionOption {
                    key: "b".to_string(),
                    label: "Option B".to_string(),
                    description: None,
                },
            ],
        );
        q.default = Some(Answer::new(AnswerValue::Selected("b".to_string())));
        assert!(matches!(
            DraftAnswer::for_question(&q),
            DraftAnswer::Selected(Some(1))
        ));
    }

    #[test]
    fn draft_answer_prepopulated_from_default_multi_select() {
        let mut q = Question::multi_select(
            "pick",
            vec![
                QuestionOption {
                    key: "a".to_string(),
                    label: "Option A".to_string(),
                    description: None,
                },
                QuestionOption {
                    key: "b".to_string(),
                    label: "Option B".to_string(),
                    description: None,
                },
            ],
        );
        q.default = Some(Answer::new(AnswerValue::MultiSelected(vec![
            "a".to_string(),
            "b".to_string(),
        ])));
        if let DraftAnswer::MultiSelected(indices) = DraftAnswer::for_question(&q) {
            assert!(indices.contains(&0));
            assert!(indices.contains(&1));
        } else {
            panic!("Expected MultiSelected");
        }
    }

    #[test]
    fn draft_answer_prepopulated_from_default_freeform() {
        let mut q = Question::freeform("name?");
        q.default = Some(Answer::new(AnswerValue::Text("Alice".to_string())));
        assert!(matches!(
            DraftAnswer::for_question(&q),
            DraftAnswer::Text(ref s) if s == "Alice"
        ));
    }

    #[test]
    fn try_set_empty_input_accepts_default_yes_no() {
        let mut q = Question::yes_no("proceed?");
        q.default = Some(Answer::new(AnswerValue::Yes));
        let interview = Interview::single(q.clone(), "test");
        let (tx, _rx) = oneshot::channel();
        let mut state = InterviewState::new(&interview, 0, tx);

        // Draft is pre-populated with the default
        assert!(matches!(
            state.draft_answers[0],
            DraftAnswer::YesNo(Some(true))
        ));

        // Empty input accepts the pre-populated default
        assert!(state.try_set_answer_from_input("", &q));
        assert!(state.validation_error.is_none());
    }

    #[test]
    fn try_set_empty_input_accepts_default_multi_choice() {
        let mut q = Question::single_select(
            "pick",
            vec![
                QuestionOption {
                    key: "a".to_string(),
                    label: "Option A".to_string(),
                    description: None,
                },
                QuestionOption {
                    key: "b".to_string(),
                    label: "Option B".to_string(),
                    description: None,
                },
            ],
        );
        q.default = Some(Answer::new(AnswerValue::Selected("b".to_string())));
        let interview = Interview::single(q.clone(), "test");
        let (tx, _rx) = oneshot::channel();
        let mut state = InterviewState::new(&interview, 0, tx);

        assert!(matches!(
            state.draft_answers[0],
            DraftAnswer::Selected(Some(1))
        ));
        assert!(state.try_set_answer_from_input("", &q));
        assert!(state.validation_error.is_none());
    }

    #[test]
    fn try_set_empty_input_accepts_focused_yes_no() {
        let q = Question::yes_no("proceed?");
        let interview = Interview::single(q.clone(), "test");
        let (tx, _rx) = oneshot::channel();
        let mut state = InterviewState::new(&interview, 0, tx);

        // No default, but focused option (Yes at index 0) is accepted
        assert!(state.try_set_answer_from_input("", &q));
        assert!(state.validation_error.is_none());
        assert!(matches!(
            state.draft_answers[0],
            DraftAnswer::YesNo(Some(true))
        ));
    }

    #[test]
    fn try_set_empty_input_accepts_focused_multi_choice() {
        let q = Question::single_select(
            "pick",
            vec![
                QuestionOption {
                    key: "a".to_string(),
                    label: "Option A".to_string(),
                    description: None,
                },
                QuestionOption {
                    key: "b".to_string(),
                    label: "Option B".to_string(),
                    description: None,
                },
            ],
        );
        let interview = Interview::single(q.clone(), "test");
        let (tx, _rx) = oneshot::channel();
        let mut state = InterviewState::new(&interview, 0, tx);

        // Focus starts at 0, so Enter selects option "a"
        assert!(state.try_set_answer_from_input("", &q));
        assert!(matches!(
            state.draft_answers[0],
            DraftAnswer::Selected(Some(0))
        ));
    }

    #[test]
    fn try_set_empty_input_still_rejects_freeform() {
        let q = Question::freeform("name?");
        let interview = Interview::single(q.clone(), "test");
        let (tx, _rx) = oneshot::channel();
        let mut state = InterviewState::new(&interview, 0, tx);

        // Freeform requires typed input
        assert!(!state.try_set_answer_from_input("", &q));
        assert!(state.validation_error.is_some());
    }

    #[test]
    fn try_set_empty_input_still_rejects_multi_select_no_selection() {
        let q = Question::multi_select(
            "pick",
            vec![QuestionOption {
                key: "a".to_string(),
                label: "Option A".to_string(),
                description: None,
            }],
        );
        let interview = Interview::single(q.clone(), "test");
        let (tx, _rx) = oneshot::channel();
        let mut state = InterviewState::new(&interview, 0, tx);

        // Multi-select requires explicit selection before submitting
        assert!(!state.try_set_answer_from_input("", &q));
        assert!(state.validation_error.is_some());
    }
}
