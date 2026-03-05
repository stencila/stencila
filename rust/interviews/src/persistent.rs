//! Persistent interviewer decorator.
//!
//! Wraps any [`Interviewer`] and persists all interview interactions
//! (question, answer, timing, options) to the `interviews` SQLite table.
//! Currently writes a completed record after the inner interviewer returns
//! an answer.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::interviewer::{Answer, AnswerValue, Interviewer, Question};

/// An interviewer decorator that persists interview records to SQLite.
///
/// Wraps an inner [`Interviewer`] and writes a row to the `interviews`
/// table after each answer is received. The `context_type` and `context_id`
/// identify the owning context (e.g., `"workflow"` + run ID, or
/// `"agent_session"` + session ID).
pub struct PersistentInterviewer {
    inner: Arc<dyn Interviewer>,
    db_conn: Arc<Mutex<stencila_db::rusqlite::Connection>>,
    context_type: String,
    context_id: String,
}

impl std::fmt::Debug for PersistentInterviewer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PersistentInterviewer")
            .field("context_type", &self.context_type)
            .field("context_id", &self.context_id)
            .finish_non_exhaustive()
    }
}

impl PersistentInterviewer {
    /// Create a new persistent interviewer.
    ///
    /// - `inner`: the underlying interviewer to delegate to
    /// - `db_conn`: shared SQLite connection (the `interviews` table must exist)
    /// - `context_type`: e.g. `"workflow"` or `"agent_session"`
    /// - `context_id`: e.g. the run ID or session ID
    pub fn new(
        inner: Arc<dyn Interviewer>,
        db_conn: Arc<Mutex<stencila_db::rusqlite::Connection>>,
        context_type: impl Into<String>,
        context_id: impl Into<String>,
    ) -> Self {
        Self {
            inner,
            db_conn,
            context_type: context_type.into(),
            context_id: context_id.into(),
        }
    }
}

#[async_trait]
impl Interviewer for PersistentInterviewer {
    async fn ask(&self, question: &Question) -> Answer {
        let started = chrono::Utc::now();
        let answer = self.inner.ask(question).await;
        let answered = chrono::Utc::now();
        let duration_ms = (answered - started).num_milliseconds();

        let answer_text = match &answer.value {
            AnswerValue::Yes => Some("yes".to_string()),
            AnswerValue::No => Some("no".to_string()),
            AnswerValue::Skipped => Some("skipped".to_string()),
            AnswerValue::Timeout => Some("timeout".to_string()),
            AnswerValue::Selected(key) => Some(key.clone()),
            AnswerValue::Text(text) => Some(text.clone()),
        };

        let selected_option = answer.selected_option.as_ref().map(|opt| opt.key.clone());
        let options_json = if question.options.is_empty() {
            None
        } else {
            let options = question
                .options
                .iter()
                .map(|opt| serde_json::json!({"key": opt.key, "label": opt.label}))
                .collect::<Vec<_>>();
            Some(serde_json::Value::Array(options).to_string())
        };

        let interview_id = uuid::Uuid::now_v7().to_string();
        let asked_at = started.to_rfc3339();
        let answered_at = answered.to_rfc3339();

        if let Ok(conn) = self
            .db_conn
            .lock()
            .map_err(|e| tracing::warn!("Poisoned DB lock: {e}"))
            && let Err(error) = conn.execute(
                "INSERT INTO interviews (
                    interview_id, context_type, context_id, node_id, question_text, question_type,
                    options, answer, selected_option, asked_at, answered_at, duration_ms
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                (
                    &interview_id,
                    &self.context_type,
                    &self.context_id,
                    &question.stage,
                    &question.text,
                    Some(question.question_type.to_string()),
                    options_json.as_deref(),
                    answer_text.as_deref(),
                    selected_option.as_deref(),
                    &asked_at,
                    Some(&answered_at),
                    Some(duration_ms),
                ),
            )
        {
            tracing::warn!("Failed to persist interview record: {error}");
        }

        answer
    }

    fn inform(&self, message: &str, stage: &str) {
        self.inner.inform(message, stage);
    }
}

/// Delete all interview records for a given context.
///
/// # Errors
///
/// Returns `rusqlite::Error` on database failure.
pub fn delete_interviews_for_context(
    conn: &stencila_db::rusqlite::Connection,
    context_type: &str,
    context_id: &str,
) -> Result<(), stencila_db::rusqlite::Error> {
    conn.execute(
        "DELETE FROM interviews WHERE context_type = ?1 AND context_id = ?2",
        (context_type, context_id),
    )?;
    Ok(())
}
