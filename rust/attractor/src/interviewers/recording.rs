//! Recording interviewer (§6.4).
//!
//! Wraps another interviewer and records all question-answer pairs.
//! Used for replay, debugging, and audit trails.

use std::sync::Mutex;

use crate::interviewer::{Answer, Interviewer, Question};

/// A recorded question-answer pair.
#[derive(Debug, Clone)]
pub struct Recording {
    /// The question that was asked.
    pub question_text: String,
    /// The answer that was given.
    pub answer: Answer,
}

/// An interviewer that records all interactions.
pub struct RecordingInterviewer {
    inner: Box<dyn Interviewer>,
    recordings: Mutex<Vec<Recording>>,
}

impl std::fmt::Debug for RecordingInterviewer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let count = self.recordings.lock().map(|r| r.len()).unwrap_or(0);
        f.debug_struct("RecordingInterviewer")
            .field("recording_count", &count)
            .finish_non_exhaustive()
    }
}

impl RecordingInterviewer {
    /// Create a new recording interviewer wrapping the given inner interviewer.
    pub fn new(inner: impl Interviewer + 'static) -> Self {
        Self {
            inner: Box::new(inner),
            recordings: Mutex::new(Vec::new()),
        }
    }

    /// Return all recorded question-answer pairs.
    #[must_use]
    pub fn recordings(&self) -> Vec<Recording> {
        self.recordings
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl Interviewer for RecordingInterviewer {
    fn ask(&self, question: &Question) -> Answer {
        let answer = self.inner.ask(question);
        let mut recs = self
            .recordings
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        recs.push(Recording {
            question_text: question.text.clone(),
            answer: answer.clone(),
        });
        answer
    }
}
