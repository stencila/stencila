//! Persistent interviewer decorator.
//!
//! Wraps any [`Interviewer`] and persists interview interactions to the
//! `interviews` and `interview_questions` SQLite tables. Writes a pending
//! row at ask-time and updates it when the answer arrives, so in-flight
//! questions survive process crashes.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::interviewer::{Answer, AnswerValue, Interview, InterviewError, Interviewer, Question};

/// An interviewer decorator that persists interview records to SQLite.
///
/// Wraps an inner [`Interviewer`] and manages the full interview lifecycle
/// in the database:
///
/// 1. On `conduct()`: writes a parent `interviews` row with `status = 'pending'`
///    and child `interview_questions` rows for each question.
/// 2. On success: updates child rows with answers, sets parent `status = 'answered'`.
/// 3. On timeout/skip: updates parent `status` accordingly.
/// 4. On error: updates parent `status = 'error'` and propagates the error.
///
/// The `context_type` and `context_id` identify the owning context (e.g.,
/// `"workflow"` + run ID, or `"agent_session"` + session ID).
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
    /// - `db_conn`: shared SQLite connection (the `interviews` and
    ///   `interview_questions` tables must exist)
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
    async fn ask(&self, question: &Question) -> Result<Answer, InterviewError> {
        let mut interview = Interview::single(question.clone());
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
        let started = chrono::Utc::now();

        // 1. Ensure interview.id is set (respect existing; generate only if empty)
        if interview.id.is_empty() {
            interview.id = uuid::Uuid::now_v7().to_string();
        }

        // 2. Ensure per-question IDs: respect pre-set IDs, generate only when absent
        let mut question_ids = Vec::with_capacity(interview.questions.len());
        for q in &mut interview.questions {
            let qid = match q.id {
                Some(ref existing) if !existing.is_empty() => existing.clone(),
                _ => {
                    let generated = uuid::Uuid::now_v7().to_string();
                    q.id = Some(generated.clone());
                    generated
                }
            };
            question_ids.push(qid);
        }

        // 3. Write pending interview + question rows
        let node_id = if interview.stage.is_empty() {
            None
        } else {
            Some(interview.stage.as_str())
        };
        let metadata_json = if interview.metadata.is_empty() {
            None
        } else {
            serde_json::to_string(&interview.metadata).ok()
        };

        insert_pending_interview(
            &self.db_conn,
            &interview.id,
            &self.context_type,
            &self.context_id,
            node_id,
            interview.stage_index,
            &started.to_rfc3339(),
            metadata_json.as_deref(),
            &interview.questions,
            &question_ids,
        )?;

        // 4. Delegate to inner interviewer
        let result = self.inner.conduct(interview).await;
        let finished = chrono::Utc::now();
        let duration_ms = (finished - started).num_milliseconds();

        match &result {
            Ok(()) => {
                // Determine the aggregate status from answers
                let status = determine_status(&interview.answers);
                update_interview_answer(
                    &self.db_conn,
                    &interview.id,
                    &interview.answers,
                    &question_ids,
                    status,
                    &finished.to_rfc3339(),
                    duration_ms,
                )?;
            }
            Err(_) => {
                update_interview_status(
                    &self.db_conn,
                    &interview.id,
                    "error",
                    &finished.to_rfc3339(),
                    duration_ms,
                )?;
            }
        }

        result
    }

    fn inform(&self, message: &str, stage: &str) {
        self.inner.inform(message, stage);
    }
}

/// Determine the aggregate status string from a list of answers.
///
/// Empty answers are treated as `"error"` because the default `conduct()`
/// implementation always populates answers via `ask()`. An empty vec after
/// `Ok(())` indicates a buggy inner interviewer rather than a user action.
fn determine_status(answers: &[Answer]) -> &'static str {
    if answers.is_empty() {
        return "error";
    }
    let all_timeout = answers.iter().all(|a| a.value == AnswerValue::Timeout);
    let all_skipped = answers.iter().all(|a| a.value == AnswerValue::Skipped);
    if all_timeout {
        "timeout"
    } else if all_skipped {
        "skipped"
    } else {
        "answered"
    }
}

/// Insert a pending interview and its questions into the database.
#[allow(clippy::too_many_arguments)]
pub fn insert_pending_interview(
    db_conn: &Arc<Mutex<stencila_db::rusqlite::Connection>>,
    interview_id: &str,
    context_type: &str,
    context_id: &str,
    node_id: Option<&str>,
    stage_index: Option<i64>,
    asked_at: &str,
    metadata: Option<&str>,
    questions: &[Question],
    question_ids: &[String],
) -> Result<(), InterviewError> {
    let conn = db_conn
        .lock()
        .map_err(|e| InterviewError::BackendFailure(format!("poisoned DB lock: {e}")))?;

    conn.execute(
        "INSERT OR IGNORE INTO interviews (
            interview_id, context_type, context_id, node_id, stage_index,
            status, asked_at, metadata
         ) VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6, ?7)",
        (
            interview_id,
            context_type,
            context_id,
            node_id,
            stage_index,
            asked_at,
            metadata,
        ),
    )
    .map_err(|e| {
        InterviewError::BackendFailure(format!("failed to insert pending interview: {e}"))
    })?;

    for (i, (q, qid)) in questions.iter().zip(question_ids.iter()).enumerate() {
        let options_json = if q.options.is_empty() {
            None
        } else {
            serde_json::to_string(&q.options).ok()
        };
        conn.execute(
            "INSERT OR IGNORE INTO interview_questions (
                question_id, interview_id, position, question_text, header,
                question_type, options
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                qid,
                interview_id,
                i as i64,
                &q.text,
                q.header.as_deref(),
                Some(q.question_type.to_string()),
                options_json.as_deref(),
            ),
        )
        .map_err(|e| {
            InterviewError::BackendFailure(format!("failed to insert interview question: {e}"))
        })?;
    }

    Ok(())
}

/// Update interview child rows with answers and set the parent status.
#[allow(clippy::too_many_arguments)]
pub fn update_interview_answer(
    db_conn: &Arc<Mutex<stencila_db::rusqlite::Connection>>,
    interview_id: &str,
    answers: &[Answer],
    question_ids: &[String],
    status: &str,
    answered_at: &str,
    duration_ms: i64,
) -> Result<(), InterviewError> {
    let conn = db_conn
        .lock()
        .map_err(|e| InterviewError::BackendFailure(format!("poisoned DB lock: {e}")))?;

    // Update each question's answer
    for (answer, qid) in answers.iter().zip(question_ids.iter()) {
        let answer_json = serde_json::to_string(&answer.value).ok();
        let selected_option = answer.selected_option.as_ref().map(|opt| opt.key.clone());
        conn.execute(
            "UPDATE interview_questions SET answer = ?1, selected_option = ?2 WHERE question_id = ?3",
            (
                answer_json.as_deref(),
                selected_option.as_deref(),
                qid,
            ),
        )
        .map_err(|e| {
            InterviewError::BackendFailure(format!("failed to update question answer: {e}"))
        })?;
    }

    // Update parent interview status
    conn.execute(
        "UPDATE interviews SET status = ?1, answered_at = ?2, duration_ms = ?3 WHERE interview_id = ?4",
        (status, answered_at, duration_ms, interview_id),
    )
    .map_err(|e| {
        InterviewError::BackendFailure(format!("failed to update interview status: {e}"))
    })?;

    Ok(())
}

/// Update an interview's status without modifying answers.
fn update_interview_status(
    db_conn: &Arc<Mutex<stencila_db::rusqlite::Connection>>,
    interview_id: &str,
    status: &str,
    answered_at: &str,
    duration_ms: i64,
) -> Result<(), InterviewError> {
    let conn = db_conn
        .lock()
        .map_err(|e| InterviewError::BackendFailure(format!("poisoned DB lock: {e}")))?;
    conn.execute(
        "UPDATE interviews SET status = ?1, answered_at = ?2, duration_ms = ?3 WHERE interview_id = ?4",
        (status, answered_at, duration_ms, interview_id),
    )
    .map_err(|e| {
        InterviewError::BackendFailure(format!("failed to update interview status: {e}"))
    })?;
    Ok(())
}

/// Delete all interview records for a given context.
///
/// Deletes from `interview_questions` via CASCADE, then from `interviews`.
///
/// # Errors
///
/// Returns `rusqlite::Error` on database failure.
pub fn delete_interviews_for_context(
    conn: &stencila_db::rusqlite::Connection,
    context_type: &str,
    context_id: &str,
) -> Result<(), stencila_db::rusqlite::Error> {
    // Delete child rows explicitly in case PRAGMA foreign_keys is off
    conn.execute(
        "DELETE FROM interview_questions WHERE interview_id IN (
            SELECT interview_id FROM interviews WHERE context_type = ?1 AND context_id = ?2
        )",
        (context_type, context_id),
    )?;
    conn.execute(
        "DELETE FROM interviews WHERE context_type = ?1 AND context_id = ?2",
        (context_type, context_id),
    )?;
    Ok(())
}

/// Look up a pending interview for a given context and node.
///
/// Used during checkpoint/resume to recover in-flight interviews. Returns
/// the `(interview_id, question_ids)` if a pending interview is found.
///
/// # Errors
///
/// Returns `InterviewError::BackendFailure` on database failure.
pub fn find_pending_interview(
    db_conn: &Arc<Mutex<stencila_db::rusqlite::Connection>>,
    context_type: &str,
    context_id: &str,
    node_id: &str,
    stage_index: Option<i64>,
) -> Result<Option<(String, Vec<String>)>, InterviewError> {
    let conn = db_conn
        .lock()
        .map_err(|e| InterviewError::BackendFailure(format!("poisoned DB lock: {e}")))?;

    let interview_id: Option<String> = if let Some(si) = stage_index {
        match conn.query_row(
            "SELECT interview_id FROM interviews
             WHERE context_type = ?1 AND context_id = ?2 AND node_id = ?3
               AND stage_index = ?4 AND status = 'pending'
             ORDER BY asked_at DESC LIMIT 1",
            (context_type, context_id, node_id, si),
            |row| row.get(0),
        ) {
            Ok(id) => Some(id),
            Err(stencila_db::rusqlite::Error::QueryReturnedNoRows) => None,
            Err(e) => {
                return Err(InterviewError::BackendFailure(format!(
                    "failed to query pending interview: {e}"
                )));
            }
        }
    } else {
        match conn.query_row(
            "SELECT interview_id FROM interviews
             WHERE context_type = ?1 AND context_id = ?2 AND node_id = ?3
               AND status = 'pending'
             ORDER BY asked_at DESC LIMIT 1",
            (context_type, context_id, node_id),
            |row| row.get(0),
        ) {
            Ok(id) => Some(id),
            Err(stencila_db::rusqlite::Error::QueryReturnedNoRows) => None,
            Err(e) => {
                return Err(InterviewError::BackendFailure(format!(
                    "failed to query pending interview: {e}"
                )));
            }
        }
    };

    let Some(iid) = interview_id else {
        return Ok(None);
    };

    let mut stmt = conn
        .prepare(
            "SELECT question_id FROM interview_questions
             WHERE interview_id = ?1 ORDER BY position",
        )
        .map_err(|e| {
            InterviewError::BackendFailure(format!("failed to query question IDs: {e}"))
        })?;

    let qids: Vec<String> = stmt
        .query_map((&iid,), |row| row.get(0))
        .map_err(|e| InterviewError::BackendFailure(format!("failed to read question IDs: {e}")))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            InterviewError::BackendFailure(format!("failed to decode question ID row: {e}"))
        })?;

    Ok(Some((iid, qids)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::INTERVIEW_MIGRATIONS;
    use crate::interviewer::{AnswerValue, QuestionOption, QuestionType};
    use crate::interviewers::AutoApproveInterviewer;

    fn setup_db() -> Arc<Mutex<stencila_db::rusqlite::Connection>> {
        let conn = stencila_db::rusqlite::Connection::open_in_memory().expect("open in-memory DB");
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .expect("enable FK");
        for m in INTERVIEW_MIGRATIONS {
            conn.execute_batch(m.sql).expect("apply migration");
        }
        Arc::new(Mutex::new(conn))
    }

    #[tokio::test]
    async fn single_question_lifecycle() {
        let db = setup_db();
        let inner = Arc::new(AutoApproveInterviewer) as Arc<dyn Interviewer>;
        let pi = PersistentInterviewer::new(inner, db.clone(), "workflow", "run-1");

        let q = Question::yes_no("Proceed?", "gate-1");
        let answer = pi.ask(&q).await.unwrap();
        assert_eq!(answer.value, AnswerValue::Yes);

        // Verify DB state
        let conn = db.lock().unwrap();
        let (status, answered_at): (String, Option<String>) = conn
            .query_row(
                "SELECT status, answered_at FROM interviews WHERE context_id = 'run-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(status, "answered");
        assert!(answered_at.is_some());

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM interview_questions WHERE interview_id IN (
                    SELECT interview_id FROM interviews WHERE context_id = 'run-1'
                )",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn conduct_multi_question() {
        let db = setup_db();
        let inner = Arc::new(AutoApproveInterviewer) as Arc<dyn Interviewer>;
        let pi = PersistentInterviewer::new(inner, db.clone(), "agent_session", "sess-1");

        let qs = vec![
            Question::yes_no("Q1?", "step"),
            Question::freeform("Q2?", "step"),
        ];
        let mut interview = Interview::batch(qs, "ask_user");
        pi.conduct(&mut interview).await.unwrap();

        assert_eq!(interview.answers.len(), 2);

        // Verify parent row
        let conn = db.lock().unwrap();
        let (status, node_id): (String, Option<String>) = conn
            .query_row(
                "SELECT status, node_id FROM interviews WHERE interview_id = ?1",
                [&interview.id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(status, "answered");
        assert_eq!(node_id.as_deref(), Some("ask_user"));

        // Verify child rows
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM interview_questions WHERE interview_id = ?1",
                [&interview.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);

        // Verify answers are persisted
        let answer_json: Option<String> = conn
            .query_row(
                "SELECT answer FROM interview_questions WHERE interview_id = ?1 AND position = 0",
                [&interview.id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(answer_json.is_some());
    }

    #[tokio::test]
    async fn question_ids_are_set() {
        let db = setup_db();
        let inner = Arc::new(AutoApproveInterviewer) as Arc<dyn Interviewer>;
        let pi = PersistentInterviewer::new(inner, db.clone(), "workflow", "run-2");

        let q = Question::yes_no("OK?", "gate");
        let mut interview = Interview::single(q);
        pi.conduct(&mut interview).await.unwrap();

        // Question should now have an ID set
        assert!(interview.questions[0].id.is_some());
        let qid = interview.questions[0].id.as_ref().unwrap();

        // ID should exist in DB
        let conn = db.lock().unwrap();
        let db_qid: String = conn
            .query_row(
                "SELECT question_id FROM interview_questions WHERE interview_id = ?1",
                [&interview.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(&db_qid, qid);
    }

    #[tokio::test]
    async fn respects_existing_interview_id() {
        let db = setup_db();
        let inner = Arc::new(AutoApproveInterviewer) as Arc<dyn Interviewer>;
        let pi = PersistentInterviewer::new(inner, db.clone(), "workflow", "run-3");

        let q = Question::yes_no("OK?", "gate");
        let mut interview = Interview::single(q);
        let original_id = interview.id.clone();
        pi.conduct(&mut interview).await.unwrap();

        assert_eq!(interview.id, original_id);
    }

    #[tokio::test]
    async fn pending_interview_visible_before_answer() {
        use crate::interviewer::InterviewError;
        use std::sync::atomic::{AtomicBool, Ordering};

        struct BlockingInterviewer {
            saw_pending: Arc<AtomicBool>,
            db: Arc<Mutex<stencila_db::rusqlite::Connection>>,
        }

        #[async_trait]
        impl Interviewer for BlockingInterviewer {
            async fn ask(&self, _question: &Question) -> Result<Answer, InterviewError> {
                // While "blocked", check that the pending row exists
                let conn = self.db.lock().unwrap();
                let status: String = conn
                    .query_row(
                        "SELECT status FROM interviews WHERE status = 'pending'",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or_default();
                self.saw_pending
                    .store(status == "pending", Ordering::Relaxed);
                Ok(Answer::new(AnswerValue::Yes))
            }
        }

        let db = setup_db();
        let saw_pending = Arc::new(AtomicBool::new(false));
        let inner = Arc::new(BlockingInterviewer {
            saw_pending: saw_pending.clone(),
            db: db.clone(),
        }) as Arc<dyn Interviewer>;
        let pi = PersistentInterviewer::new(inner, db.clone(), "workflow", "run-4");

        let q = Question::yes_no("Check?", "gate");
        pi.ask(&q).await.unwrap();

        assert!(
            saw_pending.load(Ordering::Relaxed),
            "pending row should be visible while inner interviewer is executing"
        );
    }

    #[tokio::test]
    async fn error_sets_error_status() {
        struct FailingInterviewer;

        #[async_trait]
        impl Interviewer for FailingInterviewer {
            async fn ask(&self, _q: &Question) -> Result<Answer, InterviewError> {
                Err(InterviewError::ChannelClosed)
            }
        }

        let db = setup_db();
        let inner = Arc::new(FailingInterviewer) as Arc<dyn Interviewer>;
        let pi = PersistentInterviewer::new(inner, db.clone(), "workflow", "run-err");

        let q = Question::yes_no("Fail?", "gate");
        let result = pi.ask(&q).await;
        assert!(result.is_err());

        let conn = db.lock().unwrap();
        let status: String = conn
            .query_row(
                "SELECT status FROM interviews WHERE context_id = 'run-err'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "error");
    }

    #[tokio::test]
    async fn delete_interviews_cascades() {
        let db = setup_db();
        let inner = Arc::new(AutoApproveInterviewer) as Arc<dyn Interviewer>;
        let pi = PersistentInterviewer::new(inner, db.clone(), "workflow", "run-del");

        let q = Question::yes_no("OK?", "gate");
        pi.ask(&q).await.unwrap();

        let conn = db.lock().unwrap();

        // Verify rows exist
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM interviews", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        delete_interviews_for_context(&conn, "workflow", "run-del").unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM interviews", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM interview_questions", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn find_pending_interview_returns_ids() {
        let db = setup_db();
        let question_ids = vec![uuid::Uuid::now_v7().to_string()];
        let q = Question::yes_no("Pending?", "gate-node");
        insert_pending_interview(
            &db,
            "int-pending",
            "workflow",
            "run-pend",
            Some("gate-node"),
            Some(3),
            "2024-01-01T00:00:00Z",
            None,
            &[q],
            &question_ids,
        )
        .unwrap();

        let result =
            find_pending_interview(&db, "workflow", "run-pend", "gate-node", Some(3)).unwrap();
        assert!(result.is_some());
        let (iid, qids) = result.unwrap();
        assert_eq!(iid, "int-pending");
        assert_eq!(qids.len(), 1);
        assert_eq!(qids[0], question_ids[0]);
    }

    #[tokio::test]
    async fn find_pending_interview_returns_none_when_answered() {
        let db = setup_db();
        let inner = Arc::new(AutoApproveInterviewer) as Arc<dyn Interviewer>;
        let pi = PersistentInterviewer::new(inner, db.clone(), "workflow", "run-done");

        let q = Question::yes_no("Done?", "gate");
        pi.ask(&q).await.unwrap();

        let result = find_pending_interview(&db, "workflow", "run-done", "gate", None).unwrap();
        assert!(result.is_none(), "answered interviews should not be found");
    }

    #[test]
    fn determine_status_all_timeout() {
        let answers = vec![
            Answer::new(AnswerValue::Timeout),
            Answer::new(AnswerValue::Timeout),
        ];
        assert_eq!(determine_status(&answers), "timeout");
    }

    #[test]
    fn determine_status_all_skipped() {
        let answers = vec![
            Answer::new(AnswerValue::Skipped),
            Answer::new(AnswerValue::Skipped),
        ];
        assert_eq!(determine_status(&answers), "skipped");
    }

    #[test]
    fn determine_status_mixed() {
        let answers = vec![
            Answer::new(AnswerValue::Yes),
            Answer::new(AnswerValue::Timeout),
        ];
        assert_eq!(determine_status(&answers), "answered");
    }

    #[test]
    fn determine_status_empty() {
        assert_eq!(determine_status(&[]), "error");
    }

    #[tokio::test]
    async fn multiple_choice_answer_persisted() {
        let db = setup_db();
        let inner = Arc::new(AutoApproveInterviewer) as Arc<dyn Interviewer>;
        let pi = PersistentInterviewer::new(inner, db.clone(), "workflow", "run-mc");

        let q = Question::multiple_choice(
            "Pick one:",
            vec![
                QuestionOption {
                    key: "A".into(),
                    label: "Alpha".into(),
                    description: None,
                },
                QuestionOption {
                    key: "B".into(),
                    label: "Beta".into(),
                    description: None,
                },
            ],
            "gate",
        );
        let answer = pi.ask(&q).await.unwrap();
        // AutoApproveInterviewer picks the first option
        assert_eq!(answer.value, AnswerValue::Selected("A".into()));

        let conn = db.lock().unwrap();
        let selected: Option<String> = conn
            .query_row(
                "SELECT selected_option FROM interview_questions WHERE interview_id IN (
                    SELECT interview_id FROM interviews WHERE context_id = 'run-mc'
                )",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(selected.as_deref(), Some("A"));
    }

    #[tokio::test]
    async fn stage_index_persisted_and_recoverable() {
        let db = setup_db();
        let inner = Arc::new(AutoApproveInterviewer) as Arc<dyn Interviewer>;
        let pi = PersistentInterviewer::new(inner, db.clone(), "workflow", "run-si");

        let q = Question::yes_no("OK?", "gate");
        let mut interview = Interview::single(q);
        interview.stage_index = Some(7);

        // Insert a pending row manually to simulate a crash-before-answer
        // scenario, then verify find_pending_interview can recover it.
        let qids = vec![uuid::Uuid::now_v7().to_string()];
        insert_pending_interview(
            &db,
            &interview.id,
            "workflow",
            "run-si",
            Some("gate"),
            Some(7),
            "2024-01-01T00:00:00Z",
            None,
            &interview.questions,
            &qids,
        )
        .unwrap();

        // Verify stage_index was persisted
        let conn = db.lock().unwrap();
        let si: Option<i64> = conn
            .query_row(
                "SELECT stage_index FROM interviews WHERE interview_id = ?1",
                [&interview.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(si, Some(7));
        drop(conn);

        // Recovery with matching stage_index should find it
        let result = find_pending_interview(&db, "workflow", "run-si", "gate", Some(7)).unwrap();
        assert!(result.is_some());

        // Recovery with wrong stage_index should NOT find it
        let result = find_pending_interview(&db, "workflow", "run-si", "gate", Some(99)).unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn conduct_persists_stage_index_from_interview() {
        let db = setup_db();
        let inner = Arc::new(AutoApproveInterviewer) as Arc<dyn Interviewer>;
        let pi = PersistentInterviewer::new(inner, db.clone(), "workflow", "run-si2");

        let q = Question::yes_no("OK?", "gate");
        let mut interview = Interview::single(q);
        interview.stage_index = Some(42);
        pi.conduct(&mut interview).await.unwrap();

        let conn = db.lock().unwrap();
        let si: Option<i64> = conn
            .query_row(
                "SELECT stage_index FROM interviews WHERE interview_id = ?1",
                [&interview.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(si, Some(42));
    }

    #[tokio::test]
    async fn recovered_question_ids_used_for_answer_updates() {
        let db = setup_db();

        // 1. Simulate a previously pending interview with known question IDs
        let old_interview_id = "recovered-int-1";
        let old_qid = "recovered-q-1".to_string();
        let q = Question::yes_no("Resume?", "gate");
        insert_pending_interview(
            &db,
            old_interview_id,
            "workflow",
            "run-resume",
            Some("gate"),
            Some(5),
            "2024-01-01T00:00:00Z",
            None,
            &[q.clone()],
            &[old_qid.clone()],
        )
        .unwrap();

        // 2. Now conduct with the same interview ID and pre-set question ID
        //    (simulating what WaitForHumanHandler does after recovery)
        let inner = Arc::new(AutoApproveInterviewer) as Arc<dyn Interviewer>;
        let pi = PersistentInterviewer::new(inner, db.clone(), "workflow", "run-resume");

        let mut recovered_q = q;
        recovered_q.id = Some(old_qid.clone());
        let mut interview = Interview::single(recovered_q);
        interview.id = old_interview_id.to_string();
        interview.stage_index = Some(5);
        pi.conduct(&mut interview).await.unwrap();

        // 3. Verify: the existing child row should have its answer updated
        let conn = db.lock().unwrap();
        let (answer, qid): (Option<String>, String) = conn
            .query_row(
                "SELECT answer, question_id FROM interview_questions WHERE interview_id = ?1",
                [old_interview_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(qid, old_qid, "should reuse the original question ID");
        assert!(answer.is_some(), "answer should be filled in");

        // 4. Verify: parent status should be 'answered'
        let status: String = conn
            .query_row(
                "SELECT status FROM interviews WHERE interview_id = ?1",
                [old_interview_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "answered");

        // 5. Verify: only 1 child row (no duplicate from INSERT OR IGNORE)
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM interview_questions WHERE interview_id = ?1",
                [old_interview_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn respects_preset_question_ids() {
        let db = setup_db();
        let inner = Arc::new(AutoApproveInterviewer) as Arc<dyn Interviewer>;
        let pi = PersistentInterviewer::new(inner, db.clone(), "workflow", "run-preset");

        let mut q = Question::yes_no("OK?", "gate");
        q.id = Some("my-custom-qid".to_string());
        let mut interview = Interview::single(q);
        pi.conduct(&mut interview).await.unwrap();

        assert_eq!(
            interview.questions[0].id.as_deref(),
            Some("my-custom-qid"),
            "pre-set question ID should be preserved"
        );

        let conn = db.lock().unwrap();
        let db_qid: String = conn
            .query_row(
                "SELECT question_id FROM interview_questions WHERE interview_id = ?1",
                [&interview.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(db_qid, "my-custom-qid");
    }
}
