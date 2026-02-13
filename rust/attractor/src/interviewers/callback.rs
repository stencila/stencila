//! Callback interviewer (§6.4).
//!
//! Delegates question answering to a provided callback function.
//! Useful for integrating with external systems.

use crate::interviewer::{Answer, Interviewer, Question};

/// An interviewer that delegates to a callback function.
pub struct CallbackInterviewer {
    callback: Box<dyn Fn(&Question) -> Answer + Send + Sync>,
}

impl std::fmt::Debug for CallbackInterviewer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallbackInterviewer")
            .finish_non_exhaustive()
    }
}

impl CallbackInterviewer {
    /// Create a new callback interviewer with the given function.
    pub fn new(callback: impl Fn(&Question) -> Answer + Send + Sync + 'static) -> Self {
        Self {
            callback: Box::new(callback),
        }
    }
}

impl Interviewer for CallbackInterviewer {
    fn ask(&self, question: &Question) -> Answer {
        (self.callback)(question)
    }
}
