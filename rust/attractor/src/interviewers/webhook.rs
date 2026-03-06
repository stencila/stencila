//! Webhook interviewer for remote answer collection (email, Slack, cloud).
//!
//! On `conduct()`, constructs an [`InterviewEnvelope`] from the interview
//! (including preamble and attachments), POSTs it to a configured webhook
//! URL (typically the Stencila Cloud API), then delegates to an
//! [`AwaitableInterviewer`] for the in-memory wait + DB polling.
//!
//! Two distinct URLs are involved:
//!
//! - **`webhook_url`** (outbound): where the envelope is `POSTed`. The cloud
//!   API routes it to the appropriate channel (email, Slack, etc.).
//! - **`answer_callback_url`** (inbound): serialized into the envelope so
//!   the remote system knows where to POST the [`AnswerEnvelope`] back.
//!   Typically `/api/workflows/{run_id}/interviews/{interview_id}/answers`.
//!
//! # Status
//!
//! This is a **design stub**. The HTTP POST to the webhook URL is not yet
//! implemented — it requires an HTTP client dependency (e.g., `reqwest`)
//! which is not currently in the workspace. `conduct()` currently returns
//! an error immediately. The struct, configuration, and trait wiring are
//! in place so that adding the HTTP call is a single, focused change.
//!
//! [`InterviewEnvelope`]: crate::envelope::InterviewEnvelope
//! [`AnswerEnvelope`]: crate::envelope::AnswerEnvelope

use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    envelope::InterviewEnvelope,
    interviewer::{Answer, Interview, InterviewError, Interviewer, Question},
    interviewers::AwaitableInterviewer,
};

/// Configuration for the webhook interviewer.
#[derive(Debug, Clone)]
pub struct WebhookConfig {
    /// The URL to POST [`InterviewEnvelope`] payloads to (outbound).
    ///
    /// This is the Stencila Cloud API endpoint that receives the envelope
    /// and routes it to the appropriate channel (email, Slack, etc.).
    pub webhook_url: String,
    /// The URL the external system should POST answers back to (inbound).
    ///
    /// Serialized into [`InterviewEnvelope::callback_url`] so the remote
    /// system knows where to send the [`AnswerEnvelope`]. Typically
    /// `https://server/api/workflows/{run_id}/interviews/{interview_id}/answers`.
    ///
    /// [`AnswerEnvelope`]: crate::envelope::AnswerEnvelope
    pub answer_callback_url: String,
    /// Workflow run ID (included in each envelope for routing).
    pub run_id: String,
    /// Pipeline name (included in each envelope for display).
    pub pipeline_name: String,
}

/// A remote interviewer that notifies external systems via HTTP webhook
/// and waits for answers via [`AwaitableInterviewer`].
///
/// See the [module documentation](self) for the full design.
#[derive(Debug)]
pub struct WebhookInterviewer {
    config: WebhookConfig,
    awaitable: Arc<AwaitableInterviewer>,
}

impl WebhookInterviewer {
    /// Create a new webhook interviewer.
    ///
    /// - `config`: webhook endpoint and pipeline context
    /// - `awaitable`: the shared `AwaitableInterviewer` that handles
    ///   in-memory wait and DB polling for answer delivery
    #[must_use]
    pub fn new(config: WebhookConfig, awaitable: Arc<AwaitableInterviewer>) -> Self {
        Self { config, awaitable }
    }

    /// Build the outbound envelope from an interview.
    fn build_envelope(&self, interview: &Interview) -> InterviewEnvelope {
        InterviewEnvelope::from_interview(
            interview,
            &self.config.run_id,
            &self.config.pipeline_name,
            &interview.stage,
        )
        .with_callback_url(&self.config.answer_callback_url)
    }

    /// POST the envelope to the configured webhook URL.
    ///
    /// # Current implementation
    ///
    /// This is a stub — it returns an error because the HTTP client
    /// dependency (e.g., `reqwest`) is not yet in the workspace. When an
    /// HTTP client is added, this method will perform the actual POST
    /// and the error goes away.
    fn post_envelope(&self, envelope: &InterviewEnvelope) -> Result<(), InterviewError> {
        // TODO: Replace with actual HTTP POST when reqwest or similar is available.
        // let response = client.post(&self.config.webhook_url)
        //     .json(envelope)
        //     .send()
        //     .await
        //     .map_err(|e| InterviewError::BackendFailure(format!("webhook POST failed: {e}")))?;
        // if !response.status().is_success() {
        //     return Err(InterviewError::BackendFailure(format!(
        //         "webhook returned status {}", response.status()
        //     )));
        // }
        // Ok(())

        Err(InterviewError::BackendFailure(format!(
            "webhook POST not yet implemented: would send {} question(s) for interview `{}` to {}",
            envelope.questions.len(),
            envelope.interview_id,
            self.config.webhook_url,
        )))
    }
}

#[async_trait]
impl Interviewer for WebhookInterviewer {
    async fn ask(&self, question: &Question) -> Result<Answer, InterviewError> {
        let mut interview = Interview::single(question.clone(), "");
        self.conduct(&mut interview).await?;
        interview
            .answers
            .into_iter()
            .next()
            .ok_or(InterviewError::BackendFailure(
                "no answer after conduct()".into(),
            ))
    }

    async fn conduct(&self, interview: &mut Interview) -> Result<(), InterviewError> {
        let envelope = self.build_envelope(interview);

        self.post_envelope(&envelope)?;

        self.awaitable.conduct(interview).await
    }

    fn inform(&self, message: &str, stage: &str) {
        self.awaitable.inform(message, stage);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::interviewer::Question;

    fn test_config() -> WebhookConfig {
        WebhookConfig {
            webhook_url: "https://cloud.example.com/api/interviews".into(),
            answer_callback_url:
                "https://server.example.com/api/workflows/run-1/interviews/INT/answers".into(),
            run_id: "run-1".into(),
            pipeline_name: "test-pipeline".into(),
        }
    }

    #[test]
    fn build_envelope_uses_answer_callback_url() {
        let awaitable = Arc::new(AwaitableInterviewer::new());
        let wh = WebhookInterviewer::new(test_config(), awaitable);

        let interview = Interview::single(Question::yes_no("Approve?"), "gate-1")
            .with_preamble("Review the draft.");

        let envelope = wh.build_envelope(&interview);

        assert_eq!(envelope.interview_id, interview.id);
        assert_eq!(envelope.run_id, "run-1");
        assert_eq!(envelope.pipeline_name, "test-pipeline");
        assert_eq!(envelope.node_id, "gate-1");
        assert_eq!(envelope.preamble.as_deref(), Some("Review the draft."));
        // callback_url in envelope is the *answer* callback, not the webhook URL
        assert_eq!(
            envelope.callback_url.as_deref(),
            Some("https://server.example.com/api/workflows/run-1/interviews/INT/answers")
        );
        assert_eq!(envelope.questions.len(), 1);
    }

    #[tokio::test]
    async fn conduct_returns_error_while_stub() {
        let awaitable =
            Arc::new(AwaitableInterviewer::new().with_poll_interval(Duration::from_millis(10)));
        let config = test_config();
        let wh = WebhookInterviewer::new(config, awaitable.clone());

        let mut interview = Interview::single(Question::yes_no("Proceed?"), "gate");

        let result = wh.conduct(&mut interview).await;
        let err = result.expect_err("stub should return an error");
        assert!(
            err.to_string().contains("webhook POST not yet implemented"),
            "expected stub error, got: {err}"
        );

        // Confirm nothing was registered as pending in the awaitable
        assert!(awaitable.pending_interviews().is_empty());
    }
}
