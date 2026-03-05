//! Awaitable interviewer for async answer submission.
//!
//! Supports web/API frontends where answers arrive out-of-band (HTTP/WebSocket)
//! rather than directly from stdin/stdout. Interviews are registered in a
//! thread-safe pending map and can be completed either:
//!
//! 1. In-process via [`AwaitableInterviewer::submit_answers`] (fast path), or
//! 2. Cross-process via DB polling (when `sqlite` feature is enabled).

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use crate::interviewer::{
    Answer, AnswerValue, Interview, InterviewError, Interviewer, Question, QuestionOption,
};

/// A submitted answer keyed by question ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmittedAnswer {
    /// The question being answered.
    pub question_id: String,
    /// The typed answer value.
    pub answer: Answer,
}

/// Snapshot of a pending interview for API/UI listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingInterviewSnapshot {
    /// Interview identifier.
    pub interview_id: String,
    /// Originating stage (e.g. node ID).
    pub stage: String,
    /// Questions waiting for answers.
    pub questions: Vec<Question>,
}

struct PendingInterview {
    stage: String,
    questions: Vec<Question>,
    question_ids: Vec<String>,
    responder: oneshot::Sender<Vec<SubmittedAnswer>>,
}

/// An interviewer that waits for externally submitted answers.
///
/// Designed for web/API workflows where the asking task blocks while answers
/// arrive later via HTTP/WebSocket callbacks.
#[derive(Clone)]
pub struct AwaitableInterviewer {
    pending: Arc<Mutex<HashMap<String, PendingInterview>>>,
    poll_interval: Duration,
    db_conn: Option<Arc<Mutex<stencila_db::rusqlite::Connection>>>,
}

impl std::fmt::Debug for AwaitableInterviewer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pending_count = self.pending.lock().map(|p| p.len()).unwrap_or(0);
        f.debug_struct("AwaitableInterviewer")
            .field("pending", &pending_count)
            .field("poll_interval", &self.poll_interval)
            .finish_non_exhaustive()
    }
}

impl Default for AwaitableInterviewer {
    fn default() -> Self {
        Self::new()
    }
}

impl AwaitableInterviewer {
    /// Create an awaitable interviewer with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending: Arc::new(Mutex::new(HashMap::new())),
            poll_interval: Duration::from_secs(1),
            db_conn: None,
        }
    }

    /// Set the DB polling interval.
    #[must_use]
    pub fn with_poll_interval(mut self, poll_interval: Duration) -> Self {
        self.poll_interval = poll_interval;
        self
    }

    /// Enable DB polling for cross-process completion detection.
    #[must_use]
    pub fn with_db(mut self, db_conn: Arc<Mutex<stencila_db::rusqlite::Connection>>) -> Self {
        self.db_conn = Some(db_conn);
        self
    }

    /// Return snapshots of all currently pending interviews.
    #[must_use]
    pub fn pending_interviews(&self) -> Vec<PendingInterviewSnapshot> {
        let pending = self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let mut snapshots = pending
            .iter()
            .map(|(interview_id, entry)| PendingInterviewSnapshot {
                interview_id: interview_id.clone(),
                stage: entry.stage.clone(),
                questions: entry.questions.clone(),
            })
            .collect::<Vec<_>>();

        snapshots.sort_by(|a, b| a.interview_id.cmp(&b.interview_id));
        snapshots
    }

    /// Return whether an interview is currently pending in-memory.
    #[must_use]
    pub fn has_pending(&self, interview_id: &str) -> bool {
        self.pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains_key(interview_id)
    }

    /// Submit answers for a pending interview (same-process fast path).
    ///
    /// Validates question IDs (unknown/duplicate/partial submissions are
    /// rejected) and resolves the waiting interview task.
    ///
    /// # Errors
    ///
    /// Returns [`InterviewError::BackendFailure`] for invalid payloads and
    /// unknown interview IDs, or [`InterviewError::ChannelClosed`] if the
    /// waiter is no longer listening.
    pub fn submit_answers(
        &self,
        interview_id: &str,
        answers: Vec<SubmittedAnswer>,
    ) -> Result<(), InterviewError> {
        if answers.is_empty() {
            return Err(InterviewError::BackendFailure(
                "answers payload must not be empty".into(),
            ));
        }

        let mut pending = self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let question_ids = pending
            .get(interview_id)
            .map(|entry| entry.question_ids.clone())
            .ok_or_else(|| {
                InterviewError::BackendFailure(format!(
                    "pending interview not found: {interview_id}"
                ))
            })?;

        validate_submitted_answer_ids(&question_ids, &answers)?;

        let entry = pending.remove(interview_id).ok_or_else(|| {
            InterviewError::BackendFailure(format!("pending interview not found: {interview_id}"))
        })?;

        entry
            .responder
            .send(answers)
            .map_err(|_| InterviewError::ChannelClosed)
    }

    fn clear_pending(&self, interview_id: &str) {
        let _ = self
            .pending
            .lock()
            .map(|mut pending| pending.remove(interview_id));
    }

    fn apply_submitted_answers(
        interview: &mut Interview,
        question_ids: &[String],
        submitted: Vec<SubmittedAnswer>,
    ) -> Result<(), InterviewError> {
        validate_submitted_answer_ids(question_ids, &submitted)?;

        let mut by_qid = submitted
            .into_iter()
            .map(|item| (item.question_id, item.answer))
            .collect::<HashMap<_, _>>();

        let mut ordered = Vec::with_capacity(question_ids.len());
        for qid in question_ids {
            let answer = by_qid.remove(qid).ok_or_else(|| {
                InterviewError::BackendFailure(format!("missing answer for question_id `{qid}`"))
            })?;
            ordered.push(answer);
        }

        interview.answers = ordered;
        Ok(())
    }

    fn poll_db_answers(
        &self,
        interview_id: &str,
        expected_question_count: usize,
    ) -> Result<Option<Vec<SubmittedAnswer>>, InterviewError> {
        let Some(db_conn) = &self.db_conn else {
            return Ok(None);
        };

        let conn = db_conn
            .lock()
            .map_err(|e| InterviewError::BackendFailure(format!("poisoned DB lock: {e}")))?;

        let status: Option<String> = match conn.query_row(
            "SELECT status FROM interviews WHERE interview_id = ?1",
            (interview_id,),
            |row| row.get(0),
        ) {
            Ok(status) => Some(status),
            Err(stencila_db::rusqlite::Error::QueryReturnedNoRows) => None,
            Err(error) => {
                return Err(InterviewError::BackendFailure(format!(
                    "failed querying interview status: {error}"
                )));
            }
        };

        let Some(status) = status else {
            return Ok(None);
        };

        if status == "pending" {
            return Ok(None);
        }

        if status == "error" {
            return Err(InterviewError::BackendFailure(format!(
                "interview `{interview_id}` completed with error status"
            )));
        }

        let submitted =
            read_question_answers_from_db(&conn, interview_id, &status, expected_question_count)?;

        Ok(Some(submitted))
    }


}

#[async_trait]
impl Interviewer for AwaitableInterviewer {
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
        if interview.questions.is_empty() {
            return Err(InterviewError::BackendFailure(
                "interview has no questions".into(),
            ));
        }

        if interview.id.is_empty() {
            interview.id = uuid::Uuid::now_v7().to_string();
        }

        let mut question_ids = Vec::with_capacity(interview.questions.len());
        for question in &mut interview.questions {
            let question_id = match &question.id {
                Some(existing) if !existing.is_empty() => existing.clone(),
                _ => {
                    let generated = uuid::Uuid::now_v7().to_string();
                    question.id = Some(generated.clone());
                    generated
                }
            };
            question_ids.push(question_id);
        }

        let (tx, mut rx) = oneshot::channel::<Vec<SubmittedAnswer>>();

        {
            let mut pending = self
                .pending
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if pending.contains_key(&interview.id) {
                return Err(InterviewError::BackendFailure(format!(
                    "pending interview already exists: {}",
                    interview.id
                )));
            }

            pending.insert(
                interview.id.clone(),
                PendingInterview {
                    stage: interview.stage.clone(),
                    questions: interview.questions.clone(),
                    question_ids: question_ids.clone(),
                    responder: tx,
                },
            );
        }

        let interview_id = interview.id.clone();
        let mut poll_tick = tokio::time::interval(self.poll_interval);

        let result = loop {
            tokio::select! {
                received = &mut rx => {
                    let submitted = received.map_err(|_| InterviewError::ChannelClosed)?;
                    break Self::apply_submitted_answers(interview, &question_ids, submitted);
                }
                _ = poll_tick.tick() => {
                    if let Some(submitted) = self.poll_db_answers(&interview_id, question_ids.len())? {
                        break Self::apply_submitted_answers(interview, &question_ids, submitted);
                    }
                }
            }
        };

        self.clear_pending(&interview_id);
        result
    }
}

fn read_question_answers_from_db(
    conn: &stencila_db::rusqlite::Connection,
    interview_id: &str,
    status: &str,
    expected_question_count: usize,
) -> Result<Vec<SubmittedAnswer>, InterviewError> {
    let mut stmt = conn
        .prepare(
            "SELECT question_id, answer, selected_option, options
             FROM interview_questions
             WHERE interview_id = ?1
             ORDER BY position",
        )
        .map_err(|error| {
            InterviewError::BackendFailure(format!(
                "failed preparing interview questions query: {error}"
            ))
        })?;

    let rows = stmt
        .query_map((interview_id,), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })
        .map_err(|error| {
            InterviewError::BackendFailure(format!(
                "failed querying interview question rows: {error}"
            ))
        })?;

    let mut submitted = Vec::new();
    for row in rows {
        let (question_id, answer_json, selected_option_key, options_json) =
            row.map_err(|error| {
                InterviewError::BackendFailure(format!("failed reading question row: {error}"))
            })?;

        let options = decode_options(options_json.as_deref())?;

        let value = if let Some(json) = answer_json.as_deref() {
            serde_json::from_str::<AnswerValue>(json).map_err(|error| {
                InterviewError::BackendFailure(format!(
                    "failed decoding answer JSON for question `{question_id}`: {error}"
                ))
            })?
        } else {
            status_fallback_answer_value(status, &question_id)?
        };

        let selected_option = selected_option_key.as_deref().and_then(|key| {
            options
                .iter()
                .find(|option| option.key == key)
                .cloned()
                .or_else(|| {
                    Some(QuestionOption {
                        key: key.to_string(),
                        label: key.to_string(),
                        description: None,
                    })
                })
        });

        submitted.push(SubmittedAnswer {
            question_id,
            answer: Answer {
                value,
                selected_option,
            },
        });
    }

    if submitted.len() != expected_question_count {
        return Err(InterviewError::BackendFailure(format!(
            "DB completion for interview `{interview_id}` had {} answers, expected {}",
            submitted.len(),
            expected_question_count
        )));
    }

    Ok(submitted)
}

fn validate_submitted_answer_ids(
    expected_question_ids: &[String],
    submitted: &[SubmittedAnswer],
) -> Result<(), InterviewError> {
    let expected = expected_question_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();

    let mut seen = HashSet::new();
    for item in submitted {
        if !expected.contains(item.question_id.as_str()) {
            return Err(InterviewError::BackendFailure(format!(
                "unknown question_id `{}`",
                item.question_id
            )));
        }

        if !seen.insert(item.question_id.as_str()) {
            return Err(InterviewError::BackendFailure(format!(
                "duplicate answer for question_id `{}`",
                item.question_id
            )));
        }
    }

    if seen.len() != expected.len() {
        return Err(InterviewError::BackendFailure(format!(
            "partial submission: expected {} answers, got {}",
            expected.len(),
            seen.len()
        )));
    }

    Ok(())
}

fn decode_options(options_json: Option<&str>) -> Result<Vec<QuestionOption>, InterviewError> {
    let Some(json) = options_json else {
        return Ok(Vec::new());
    };

    serde_json::from_str::<Vec<QuestionOption>>(json).map_err(|error| {
        InterviewError::BackendFailure(format!("failed decoding question options JSON: {error}"))
    })
}

fn status_fallback_answer_value(
    status: &str,
    question_id: &str,
) -> Result<AnswerValue, InterviewError> {
    match status {
        "timeout" => Ok(AnswerValue::Timeout),
        "skipped" => Ok(AnswerValue::Skipped),
        "answered" => Err(InterviewError::BackendFailure(format!(
            "missing answer value for answered question `{question_id}`"
        ))),
        other => Err(InterviewError::BackendFailure(format!(
            "unsupported interview status `{other}` while reconstructing answer"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn submit_answers_unblocks_conduct() {
        let interviewer =
            Arc::new(AwaitableInterviewer::new().with_poll_interval(Duration::from_millis(10)));

        let mut question = Question::yes_no("Proceed?", "gate");
        question.id = Some("q-1".into());
        let mut interview = Interview::single(question);
        interview.id = "int-1".into();

        let interviewer_for_task = interviewer.clone();
        let handle = tokio::spawn(async move {
            let mut local = interview;
            interviewer_for_task
                .conduct(&mut local)
                .await
                .map(|()| local)
        });

        tokio::time::sleep(Duration::from_millis(25)).await;

        let pending = interviewer.pending_interviews();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].interview_id, "int-1");

        interviewer
            .submit_answers(
                "int-1",
                vec![SubmittedAnswer {
                    question_id: "q-1".into(),
                    answer: Answer::new(AnswerValue::Yes),
                }],
            )
            .expect("submit answers should succeed");

        let completed = handle
            .await
            .expect("task should complete")
            .expect("conduct should succeed");

        assert_eq!(completed.answers.len(), 1);
        assert_eq!(completed.answers[0].value, AnswerValue::Yes);
        assert!(interviewer.pending_interviews().is_empty());
    }

    #[tokio::test]
    async fn submit_answers_rejects_partial_payloads() {
        let interviewer =
            Arc::new(AwaitableInterviewer::new().with_poll_interval(Duration::from_millis(10)));

        let mut q1 = Question::yes_no("Q1?", "gate");
        q1.id = Some("q-1".into());
        let mut q2 = Question::freeform("Q2?", "gate");
        q2.id = Some("q-2".into());

        let mut interview = Interview::batch(vec![q1, q2], "gate");
        interview.id = "int-2".into();

        let interviewer_for_task = interviewer.clone();
        let handle = tokio::spawn(async move {
            let mut local = interview;
            interviewer_for_task
                .conduct(&mut local)
                .await
                .map(|()| local)
        });

        tokio::time::sleep(Duration::from_millis(25)).await;

        let error = interviewer
            .submit_answers(
                "int-2",
                vec![SubmittedAnswer {
                    question_id: "q-1".into(),
                    answer: Answer::new(AnswerValue::Yes),
                }],
            )
            .expect_err("partial submissions should fail");

        match error {
            InterviewError::BackendFailure(message) => {
                assert!(message.contains("partial submission"));
            }
            other => panic!("unexpected error: {other}"),
        }

        interviewer
            .submit_answers(
                "int-2",
                vec![
                    SubmittedAnswer {
                        question_id: "q-1".into(),
                        answer: Answer::new(AnswerValue::Yes),
                    },
                    SubmittedAnswer {
                        question_id: "q-2".into(),
                        answer: Answer::new(AnswerValue::Text("ok".into())),
                    },
                ],
            )
            .expect("valid submission should succeed");

        let completed = handle
            .await
            .expect("task should join")
            .expect("conduct should succeed");
        assert_eq!(completed.answers.len(), 2);
    }

    #[tokio::test]
    async fn db_polling_reconstructs_answers() {
        let conn = stencila_db::rusqlite::Connection::open_in_memory().expect("open in-memory db");
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .expect("enable foreign keys");
        for migration in stencila_interviews::INTERVIEW_MIGRATIONS {
            conn.execute_batch(migration.sql)
                .expect("apply interview migration");
        }

        let db = Arc::new(Mutex::new(conn));

        {
            let conn = db.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            conn.execute(
                "INSERT INTO interviews (interview_id, context_type, context_id, node_id, status, asked_at)
                 VALUES (?1, 'workflow', 'run-1', 'gate', 'pending', strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
                ("int-db",),
            )
            .expect("insert interview");
            conn.execute(
                "INSERT INTO interview_questions (
                    question_id, interview_id, position, question_text, question_type
                 ) VALUES (?1, ?2, 0, 'Proceed?', 'YES_NO')",
                ("q-db", "int-db"),
            )
            .expect("insert question");
        }

        let interviewer = Arc::new(
            AwaitableInterviewer::new()
                .with_db(db.clone())
                .with_poll_interval(Duration::from_millis(10)),
        );

        let mut question = Question::yes_no("Proceed?", "gate");
        question.id = Some("q-db".into());
        let mut interview = Interview::single(question);
        interview.id = "int-db".into();

        let interviewer_for_task = interviewer.clone();
        let handle = tokio::spawn(async move {
            let mut local = interview;
            interviewer_for_task
                .conduct(&mut local)
                .await
                .map(|()| local)
        });

        tokio::time::sleep(Duration::from_millis(40)).await;

        {
            let conn = db.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            conn.execute(
                "UPDATE interview_questions SET answer = ?1 WHERE question_id = ?2",
                (r#"{"type":"YES"}"#, "q-db"),
            )
            .expect("update question answer");
            conn.execute(
                "UPDATE interviews
                 SET status = 'answered',
                     answered_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                     duration_ms = 10
                 WHERE interview_id = ?1",
                ("int-db",),
            )
            .expect("mark interview answered");
        }

        let completed = handle
            .await
            .expect("task should join")
            .expect("conduct should succeed");

        assert_eq!(completed.answers.len(), 1);
        assert_eq!(completed.answers[0].value, AnswerValue::Yes);
        assert!(interviewer.pending_interviews().is_empty());
    }
}
