use std::collections::HashSet;

use stencila_attractor::interviewer::{
    Answer, AnswerValue, Interview, InterviewError, Interviewer, Question, QuestionType,
    parse_answer_text,
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
    pub fn for_question(question: &Question) -> Self {
        match question.question_type {
            QuestionType::Freeform => Self::Pending,
            QuestionType::YesNo | QuestionType::Confirmation => Self::YesNo(None),
            QuestionType::MultipleChoice => Self::Selected(None),
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

        Self {
            interview_id: interview.id.clone(),
            msg_index,
            current_question: 0,
            draft_answers,
            answer_tx: Some(answer_tx),
            validation_error: None,
        }
    }

    /// Try to set the answer for the current question from input text.
    ///
    /// Returns `true` if the input was valid for this question type,
    /// `false` if validation failed (with `validation_error` set).
    pub fn try_set_answer_from_input(&mut self, input: &str, question: &Question) -> bool {
        let trimmed = input.trim();

        match question.question_type {
            QuestionType::Freeform => {
                if trimmed.is_empty() {
                    self.validation_error = Some("Enter your answer".to_string());
                    return false;
                }
                self.draft_answers[self.current_question] = DraftAnswer::Text(trimmed.to_string());
                self.validation_error = None;
                true
            }
            QuestionType::YesNo | QuestionType::Confirmation => {
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
                    self.validation_error = Some("Enter y or n".to_string());
                    false
                }
            }
            QuestionType::MultipleChoice => {
                if let Some(idx) = question
                    .options
                    .iter()
                    .position(|o| o.key.eq_ignore_ascii_case(trimmed))
                {
                    self.draft_answers[self.current_question] = DraftAnswer::Selected(Some(idx));
                    self.validation_error = None;
                    true
                } else if let Some(idx) = question
                    .options
                    .iter()
                    .position(|o| o.label.eq_ignore_ascii_case(trimmed))
                {
                    self.draft_answers[self.current_question] = DraftAnswer::Selected(Some(idx));
                    self.validation_error = None;
                    true
                } else {
                    let keys: Vec<&str> = question.options.iter().map(|o| o.key.as_str()).collect();
                    self.validation_error = Some(format!("Choose: {}", keys.join(", ")));
                    false
                }
            }
            QuestionType::MultiSelect => {
                let parts: Vec<&str> = trimmed
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .collect();
                if parts.is_empty() {
                    self.validation_error = Some("Select at least one option".to_string());
                    return false;
                }
                let mut indices = HashSet::new();
                for part in &parts {
                    if let Some(idx) = question
                        .options
                        .iter()
                        .position(|o| o.key.eq_ignore_ascii_case(part))
                    {
                        indices.insert(idx);
                    } else if let Some(idx) = question
                        .options
                        .iter()
                        .position(|o| o.label.eq_ignore_ascii_case(part))
                    {
                        indices.insert(idx);
                    } else {
                        let keys: Vec<&str> =
                            question.options.iter().map(|o| o.key.as_str()).collect();
                        self.validation_error = Some(format!(
                            "Unknown option \"{part}\". Choose from: {}",
                            keys.join(", ")
                        ));
                        return false;
                    }
                }
                self.draft_answers[self.current_question] = DraftAnswer::MultiSelected(indices);
                self.validation_error = None;
                true
            }
        }
    }

    /// Advance to the next question. Returns `true` if all questions are done.
    pub fn advance(&mut self) -> bool {
        self.current_question += 1;
        self.validation_error = None;
        self.current_question >= self.draft_answers.len()
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
    pub fn finalize_answers(&self, questions: &[Question]) -> Vec<Answer> {
        self.draft_answers
            .iter()
            .zip(questions)
            .map(|(draft, q)| draft.to_answer(q))
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

        let q = Question::confirmation("test");
        assert!(matches!(
            DraftAnswer::for_question(&q),
            DraftAnswer::YesNo(None)
        ));

        let q = Question::multiple_choice(
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
    fn try_set_multiple_choice_answer() {
        let q = Question::multiple_choice(
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
        let q = Question::multiple_choice(
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
}
