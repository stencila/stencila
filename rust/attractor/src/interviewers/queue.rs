//! Queue interviewer (§6.4).
//!
//! Reads answers from a pre-filled queue. Returns SKIPPED when
//! the queue is exhausted. Used for deterministic testing and replay.

use std::sync::Mutex;

use crate::interviewer::{Answer, AnswerValue, Interviewer, Question};

/// An interviewer that dequeues pre-filled answers.
///
/// Thread-safe: the internal queue is protected by a [`Mutex`].
pub struct QueueInterviewer {
    answers: Mutex<Vec<Answer>>,
}

impl std::fmt::Debug for QueueInterviewer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.answers.lock().map(|q| q.len()).unwrap_or(0);
        f.debug_struct("QueueInterviewer")
            .field("remaining", &len)
            .finish()
    }
}

impl QueueInterviewer {
    /// Create a new queue interviewer with the given answers.
    ///
    /// Answers are dequeued in FIFO order (first answer is returned first).
    #[must_use]
    pub fn new(answers: Vec<Answer>) -> Self {
        // Reverse so we can pop from the end efficiently (FIFO order).
        let mut reversed = answers;
        reversed.reverse();
        Self {
            answers: Mutex::new(reversed),
        }
    }

    /// Return how many answers remain in the queue.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.answers.lock().map(|q| q.len()).unwrap_or(0)
    }
}

impl Interviewer for QueueInterviewer {
    fn ask(&self, _question: &Question) -> Answer {
        let mut queue = self
            .answers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        queue
            .pop()
            .unwrap_or_else(|| Answer::new(AnswerValue::Skipped))
    }
}
