//! Auto-approve interviewer (§6.4).
//!
//! Always selects YES for yes/no and confirmation questions,
//! the first option for multiple choice, and `"auto-approved"` for freeform.

use crate::interviewer::{Answer, AnswerValue, Interviewer, Question, QuestionType};

/// An interviewer that automatically approves all questions.
///
/// Used for automated testing and CI/CD pipelines where no human
/// is available.
#[derive(Debug, Clone, Copy, Default)]
pub struct AutoApproveInterviewer;

impl Interviewer for AutoApproveInterviewer {
    fn ask(&self, question: &Question) -> Answer {
        match question.question_type {
            QuestionType::YesNo | QuestionType::Confirmation => Answer::new(AnswerValue::Yes),
            QuestionType::MultipleChoice => {
                if let Some(first) = question.options.first() {
                    Answer::with_option(AnswerValue::Selected(first.key.clone()), first.clone())
                } else {
                    Answer::new(AnswerValue::Text("auto-approved".into()))
                }
            }
            QuestionType::Freeform => Answer::new(AnswerValue::Text("auto-approved".into())),
        }
    }
}
