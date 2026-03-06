//! Wire-format DTOs for transmitting interviews and answers across process
//! and network boundaries.
//!
//! These are the stable JSON contract for external systems (cloud API, email,
//! Slack). Local interviewers (TUI, CLI) do not use envelopes — they interact
//! with the [`Interviewer`] trait directly.
//!
//! [`Interviewer`]: crate::interviewer::Interviewer

use serde::{Deserialize, Serialize};

use crate::interviewer::{Attachment, Question};
pub use crate::interviewers::awaitable::SubmittedAnswer;

/// Outbound envelope: sent to external systems when an interview
/// is waiting for a human response.
///
/// Carries the full interview context — preamble text, attached artifacts
/// with download URLs, and typed questions — so external systems can render
/// rich notifications without querying back for additional data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterviewEnvelope {
    /// Unique interview identifier.
    pub interview_id: String,
    /// Workflow run ID (for routing answers back).
    pub run_id: String,
    /// Pipeline name (for display in notifications).
    pub pipeline_name: String,
    /// Originating node ID within the pipeline.
    pub node_id: String,
    /// Introductory text displayed before the questions.
    /// Provides framing context — e.g., "Here's the quarterly
    /// report draft."
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preamble: Option<String>,
    /// Artifacts attached for the human to review alongside the
    /// questions. URLs should be populated by the server layer
    /// before posting the envelope externally.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,
    /// The questions to present.
    pub questions: Vec<Question>,
    /// URL for the external system to POST answers back to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
}

impl InterviewEnvelope {
    /// Construct an envelope from an [`Interview`] and pipeline context.
    ///
    /// Copies `preamble`, `attachments`, and `questions` from the interview.
    ///
    /// [`Interview`]: crate::interviewer::Interview
    #[must_use]
    pub fn from_interview(
        interview: &crate::interviewer::Interview,
        run_id: impl Into<String>,
        pipeline_name: impl Into<String>,
        node_id: impl Into<String>,
    ) -> Self {
        Self {
            interview_id: interview.id.clone(),
            run_id: run_id.into(),
            pipeline_name: pipeline_name.into(),
            node_id: node_id.into(),
            preamble: interview.preamble.clone(),
            attachments: interview.attachments.clone(),
            questions: interview.questions.clone(),
            callback_url: None,
        }
    }

    /// Set the callback URL for the external system to POST answers back to.
    #[must_use]
    pub fn with_callback_url(mut self, url: impl Into<String>) -> Self {
        self.callback_url = Some(url.into());
        self
    }
}

/// Inbound envelope: received from external systems with answers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerEnvelope {
    /// Interview being answered.
    pub interview_id: String,
    /// Per-question answers.
    pub answers: Vec<SubmittedAnswer>,
    /// Identity of the responder (email, Slack user ID, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub responder: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interviewer::{Answer, AnswerValue, Interview, Question};

    #[test]
    fn envelope_from_interview_copies_fields() {
        let att = Attachment {
            id: "att-1".into(),
            filename: "report.pdf".into(),
            media_type: "application/pdf".into(),
            url: Some("https://cdn.example.com/report.pdf".into()),
            size_bytes: Some(1024),
            description: None,
        };
        let interview = Interview::single(Question::yes_no("Approve?"), "review-gate")
            .with_preamble("Please review the quarterly report.")
            .with_attachment(att.clone());

        let envelope =
            InterviewEnvelope::from_interview(&interview, "run-42", "quarterly-report", "gate-1");

        assert_eq!(envelope.interview_id, interview.id);
        assert_eq!(envelope.run_id, "run-42");
        assert_eq!(envelope.pipeline_name, "quarterly-report");
        assert_eq!(envelope.node_id, "gate-1");
        assert_eq!(
            envelope.preamble.as_deref(),
            Some("Please review the quarterly report.")
        );
        assert_eq!(envelope.attachments.len(), 1);
        assert_eq!(envelope.attachments[0], att);
        assert_eq!(envelope.questions.len(), 1);
        assert!(envelope.callback_url.is_none());
    }

    #[test]
    fn envelope_with_callback_url() {
        let interview = Interview::single(Question::yes_no("OK?"), "gate");
        let envelope = InterviewEnvelope::from_interview(&interview, "run-1", "pipe", "gate")
            .with_callback_url("https://api.example.com/answers");
        assert_eq!(
            envelope.callback_url.as_deref(),
            Some("https://api.example.com/answers")
        );
    }

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn interview_envelope_serde_roundtrip() -> TestResult {
        let interview = Interview::single(Question::yes_no("Approve?"), "review-gate")
            .with_preamble("Review this.");
        let envelope =
            InterviewEnvelope::from_interview(&interview, "run-1", "pipeline-a", "gate-1")
                .with_callback_url("https://example.com/callback");

        let json = serde_json::to_string(&envelope)?;
        let envelope2: InterviewEnvelope = serde_json::from_str(&json)?;
        assert_eq!(envelope2.interview_id, envelope.interview_id);
        assert_eq!(envelope2.run_id, "run-1");
        assert_eq!(envelope2.pipeline_name, "pipeline-a");
        assert_eq!(envelope2.node_id, "gate-1");
        assert_eq!(envelope2.preamble, envelope.preamble);
        assert_eq!(envelope2.questions.len(), 1);
        assert_eq!(
            envelope2.callback_url.as_deref(),
            Some("https://example.com/callback")
        );
        Ok(())
    }

    #[test]
    fn interview_envelope_optional_fields_omitted() -> TestResult {
        let interview = Interview::single(Question::yes_no("OK?"), "gate");
        let envelope = InterviewEnvelope::from_interview(&interview, "run-1", "pipe", "gate");
        let json = serde_json::to_string(&envelope)?;
        assert!(!json.contains("preamble"));
        assert!(!json.contains("attachments"));
        assert!(!json.contains("callback_url"));
        Ok(())
    }

    #[test]
    fn answer_envelope_serde_roundtrip() -> TestResult {
        let envelope = AnswerEnvelope {
            interview_id: "int-1".into(),
            answers: vec![SubmittedAnswer {
                question_id: "q-1".into(),
                answer: Answer::new(AnswerValue::Yes),
            }],
            responder: Some("user@example.com".into()),
        };

        let json = serde_json::to_string(&envelope)?;
        let envelope2: AnswerEnvelope = serde_json::from_str(&json)?;
        assert_eq!(envelope2.interview_id, "int-1");
        assert_eq!(envelope2.answers.len(), 1);
        assert_eq!(envelope2.answers[0].question_id, "q-1");
        assert_eq!(envelope2.answers[0].answer.value, AnswerValue::Yes);
        assert_eq!(envelope2.responder.as_deref(), Some("user@example.com"));
        Ok(())
    }

    #[test]
    fn answer_envelope_optional_responder_omitted() -> TestResult {
        let envelope = AnswerEnvelope {
            interview_id: "int-1".into(),
            answers: vec![SubmittedAnswer {
                question_id: "q-1".into(),
                answer: Answer::new(AnswerValue::No),
            }],
            responder: None,
        };
        let json = serde_json::to_string(&envelope)?;
        assert!(!json.contains("responder"));
        Ok(())
    }

    #[test]
    fn submitted_answer_serde_roundtrip() -> TestResult {
        let sa = SubmittedAnswer {
            question_id: "q-42".into(),
            answer: Answer::new(AnswerValue::Text("Some feedback".into())),
        };
        let json = serde_json::to_string(&sa)?;
        let sa2: SubmittedAnswer = serde_json::from_str(&json)?;
        assert_eq!(sa2.question_id, "q-42");
        assert_eq!(sa2.answer.value, AnswerValue::Text("Some feedback".into()));
        Ok(())
    }

    #[test]
    fn envelope_multi_question_with_attachments() -> TestResult {
        let att = Attachment {
            id: "att-1".into(),
            filename: "draft.docx".into(),
            media_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                .into(),
            url: None,
            size_bytes: None,
            description: Some("Draft report".into()),
        };
        let qs = vec![
            Question::yes_no("Formatting OK?"),
            Question::yes_no("Add TOC?"),
        ];
        let interview = Interview::batch(qs, "review")
            .with_preamble("Here's the report draft.")
            .with_attachment(att);
        let envelope =
            InterviewEnvelope::from_interview(&interview, "run-1", "report-gen", "review-gate");

        assert_eq!(envelope.questions.len(), 2);
        assert_eq!(envelope.attachments.len(), 1);
        assert_eq!(
            envelope.preamble.as_deref(),
            Some("Here's the report draft.")
        );

        let answer_envelope = AnswerEnvelope {
            interview_id: envelope.interview_id.clone(),
            answers: vec![
                SubmittedAnswer {
                    question_id: "q-1".into(),
                    answer: Answer::new(AnswerValue::Yes),
                },
                SubmittedAnswer {
                    question_id: "q-2".into(),
                    answer: Answer::new(AnswerValue::No),
                },
            ],
            responder: Some("reviewer@corp.com".into()),
        };

        let json = serde_json::to_string(&answer_envelope)?;
        let decoded: AnswerEnvelope = serde_json::from_str(&json)?;
        assert_eq!(decoded.answers.len(), 2);
        Ok(())
    }
}
