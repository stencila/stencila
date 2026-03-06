//! A decorator that overrides the `Interview.stage` field.
//!
//! Used when an `ask_user` tool runs inside a workflow pipeline node: the
//! tool creates interviews with `stage = "ask_user"`, but for persistence
//! and audit the stage should be the actual pipeline node ID (e.g.,
//! `"draft-report"`). This decorator wraps the real interviewer and
//! rewrites the stage before delegating.

use std::sync::Arc;

use async_trait::async_trait;

use crate::interviewer::{Answer, InterviewError, Interview, Interviewer, Question};

pub struct StageOverrideInterviewer {
    inner: Arc<dyn Interviewer>,
    stage: String,
}

impl StageOverrideInterviewer {
    pub fn new(inner: Arc<dyn Interviewer>, stage: impl Into<String>) -> Self {
        Self {
            inner,
            stage: stage.into(),
        }
    }
}

#[async_trait]
impl Interviewer for StageOverrideInterviewer {
    async fn ask(&self, question: &Question) -> Result<Answer, InterviewError> {
        self.inner.ask(question).await
    }

    async fn conduct(&self, interview: &mut Interview) -> Result<(), InterviewError> {
        interview.stage = self.stage.clone();
        self.inner.conduct(interview).await
    }

    fn inform(&self, message: &str, stage: &str) {
        self.inner.inform(message, stage);
    }
}
