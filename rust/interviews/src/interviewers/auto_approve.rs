//! Auto-approve interviewer (§6.4).
//!
//! Always selects YES for yes/no and confirmation questions,
//! the first option for multiple choice, and `"auto-approved"` for freeform.

use async_trait::async_trait;

use crate::interviewer::{
    Answer, AnswerValue, InterviewError, Interviewer, Question, QuestionType,
};

/// An interviewer that automatically approves all questions.
///
/// Used for automated testing and CI/CD pipelines where no human
/// is available.
#[derive(Debug, Clone, Copy, Default)]
pub struct AutoApproveInterviewer;

#[async_trait]
impl Interviewer for AutoApproveInterviewer {
    async fn ask(&self, question: &Question) -> Result<Answer, InterviewError> {
        Ok(match question.question_type {
            QuestionType::YesNo | QuestionType::Confirmation => Answer::new(AnswerValue::Yes),
            QuestionType::MultipleChoice => {
                if let Some(first) = question.options.first() {
                    Answer::with_option(AnswerValue::Selected(first.key.clone()), first.clone())
                } else {
                    Answer::new(AnswerValue::Text("auto-approved".into()))
                }
            }
            // Selects *all* options (most permissive) — mirrors selecting
            // the first option for single-select MultipleChoice.
            QuestionType::MultiSelect => {
                let keys: Vec<String> = question.options.iter().map(|o| o.key.clone()).collect();
                if keys.is_empty() {
                    Answer::new(AnswerValue::Text("auto-approved".into()))
                } else {
                    Answer::new(AnswerValue::MultiSelected(keys))
                }
            }
            QuestionType::Freeform => Answer::new(AnswerValue::Text("auto-approved".into())),
        })
    }
}
