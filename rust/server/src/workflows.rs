use std::{collections::HashMap, sync::Arc};

use axum::{
    Json, Router,
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use stencila_attractor::{
    events::{EventEmitter, ObserverEmitter, PipelineEvent},
    interviewers::SubmittedAnswer,
};
use stencila_interviews::interviewer::{Answer, AnswerValue, Question, QuestionOption};

use crate::server::ServerState;

/// Envelope pushed to workflow interview websocket clients.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkflowInterviewEventEnvelope {
    QuestionAsked {
        interview_id: String,
        node_id: String,
        question: Question,
    },
    AnswerReceived {
        interview_id: String,
        node_id: String,
    },
    TimedOut {
        interview_id: String,
        node_id: String,
    },
}

/// Install workflow interview API routes.
pub fn router() -> Router<ServerState> {
    Router::new()
        .route("/websocket", get(workflow_websocket_handler))
        .route("/{run_id}/interviews", get(list_pending_interviews))
        .route(
            "/{run_id}/interviews/{interview_id}/answers",
            post(submit_interview_answers),
        )
}

/// Create a workflow event emitter that forwards interview events to clients.
#[must_use]
pub fn interview_event_emitter(
    sender: tokio::sync::broadcast::Sender<WorkflowInterviewEventEnvelope>,
) -> Arc<dyn EventEmitter> {
    Arc::new(ObserverEmitter::new(move |event: &PipelineEvent| {
        if let Some(envelope) = pipeline_event_to_envelope(event) {
            let _ = sender.send(envelope);
        }
    }))
}

async fn workflow_websocket_handler(
    State(state): State<ServerState>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let Some(sender) = &state.workflow_notify else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "workflow notifications are not configured".to_string(),
        ));
    };

    let receiver = sender.subscribe();
    Ok(ws.on_upgrade(move |socket| workflow_socket(socket, receiver)))
}

async fn workflow_socket(
    mut socket: WebSocket,
    mut receiver: tokio::sync::broadcast::Receiver<WorkflowInterviewEventEnvelope>,
) {
    while let Ok(event) = receiver.recv().await {
        let Ok(json) = serde_json::to_string(&event) else {
            continue;
        };

        if socket.send(Message::Text(json.into())).await.is_err() {
            break;
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct PendingInterviewResponse {
    interview_id: String,
    context_type: String,
    context_id: String,
    node_id: Option<String>,
    stage_index: Option<i64>,
    asked_at: String,
    questions: Vec<PendingQuestionResponse>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct PendingQuestionResponse {
    question_id: String,
    position: i64,
    question: Question,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct SubmitAnswersRequest {
    answers: Vec<SubmittedAnswerPayload>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct SubmittedAnswerPayload {
    question_id: String,
    answer: Answer,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
struct SubmitAnswersResponse {
    interview_id: String,
    accepted: usize,
    mode: &'static str,
}

async fn list_pending_interviews(
    State(state): State<ServerState>,
    Path(run_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let db = open_workspace_db(&state).await?;
    let conn = db
        .connection()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let mut stmt = conn
        .prepare(
            "SELECT
                i.interview_id,
                i.context_type,
                i.context_id,
                i.node_id,
                i.stage_index,
                i.asked_at,
                q.question_id,
                q.position,
                q.question_text,
                q.header,
                q.question_type,
                q.options
             FROM interviews i
             JOIN interview_questions q ON q.interview_id = i.interview_id
             WHERE i.context_type = 'workflow'
               AND i.context_id = ?1
               AND i.status = 'pending'
             ORDER BY i.asked_at, q.position",
        )
        .map_err(internal)?;

    let rows = stmt
        .query_map((run_id.as_str(),), |row| {
            Ok(DbPendingRow {
                interview_id: row.get(0)?,
                context_type: row.get(1)?,
                context_id: row.get(2)?,
                node_id: row.get(3)?,
                stage_index: row.get(4)?,
                asked_at: row.get(5)?,
                question_id: row.get(6)?,
                position: row.get(7)?,
                question_text: row.get(8)?,
                header: row.get(9)?,
                question_type: row.get(10)?,
                options: row.get(11)?,
            })
        })
        .map_err(internal)?;

    let mut interviews: HashMap<String, PendingInterviewResponse> = HashMap::new();

    for row in rows {
        let row = row.map_err(internal)?;

        let question = Question {
            id: Some(row.question_id.clone()),
            text: row.question_text,
            header: row.header,
            question_type: parse_question_type(row.question_type.as_deref()),
            options: parse_options(row.options.as_deref())
                .map_err(|error| bad_request(format!("invalid options JSON: {error}")))?,
            default: None,
            timeout_seconds: None,
            stage: row.node_id.clone().unwrap_or_default(),
            metadata: Default::default(),
        };

        let entry = interviews
            .entry(row.interview_id.clone())
            .or_insert_with(|| PendingInterviewResponse {
                interview_id: row.interview_id.clone(),
                context_type: row.context_type.clone(),
                context_id: row.context_id.clone(),
                node_id: row.node_id.clone(),
                stage_index: row.stage_index,
                asked_at: row.asked_at.clone(),
                questions: Vec::new(),
            });

        entry.questions.push(PendingQuestionResponse {
            question_id: row.question_id,
            position: row.position,
            question,
        });
    }

    let mut list = interviews.into_values().collect::<Vec<_>>();
    list.sort_by(|a, b| {
        a.asked_at
            .cmp(&b.asked_at)
            .then(a.interview_id.cmp(&b.interview_id))
    });

    Ok(Json(list))
}

async fn submit_interview_answers(
    State(state): State<ServerState>,
    Path((run_id, interview_id)): Path<(String, String)>,
    Json(payload): Json<SubmitAnswersRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if payload.answers.is_empty() {
        return Err(bad_request("answers payload must not be empty"));
    }

    let submitted = payload
        .answers
        .iter()
        .map(|item| SubmittedAnswer {
            question_id: item.question_id.clone(),
            answer: item.answer.clone(),
        })
        .collect::<Vec<_>>();

    if let Some(interviewer) = &state.workflow_interviewer {
        match interviewer.submit_answers(&interview_id, submitted.clone()) {
            Ok(()) => {
                return Ok(Json(SubmitAnswersResponse {
                    interview_id,
                    accepted: submitted.len(),
                    mode: "in_process",
                }));
            }
            Err(stencila_interviews::interviewer::InterviewError::BackendFailure(message))
                if message.contains("pending interview not found") => {}
            Err(error) => return Err(bad_request(format!("invalid answers payload: {error}"))),
        }
    }

    // Cross-process fallback: write completion directly to DB.
    let db = open_workspace_db(&state).await?;
    let conn = db
        .connection()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let tx = conn.unchecked_transaction().map_err(internal)?;

    let status: String = tx
        .query_row(
            "SELECT status FROM interviews
             WHERE interview_id = ?1 AND context_type = 'workflow' AND context_id = ?2",
            (&interview_id, &run_id),
            |row| row.get(0),
        )
        .map_err(|error| match error {
            stencila_db::rusqlite::Error::QueryReturnedNoRows => not_found(format!(
                "interview `{interview_id}` not found for run `{run_id}`"
            )),
            _ => internal(error),
        })?;

    if status != "pending" {
        return Err(conflict(format!(
            "interview `{interview_id}` is not pending (status={status})"
        )));
    }

    let (expected_ids, options_by_id) = {
        let mut stmt = tx
            .prepare(
                "SELECT question_id, options
                 FROM interview_questions
                 WHERE interview_id = ?1
                 ORDER BY position",
            )
            .map_err(internal)?;

        let rows = stmt
            .query_map((&interview_id,), |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
            })
            .map_err(internal)?;

        let mut ids = Vec::new();
        let mut opts = HashMap::new();
        for row in rows {
            let (qid, options_json) = row.map_err(internal)?;
            ids.push(qid.clone());
            let options = parse_options(options_json.as_deref())
                .map_err(|error| internal(format!("invalid options JSON in DB: {error}")))?;
            opts.insert(qid, options);
        }
        (ids, opts)
    };

    if expected_ids.is_empty() {
        return Err(internal(format!(
            "interview `{interview_id}` has no interview_questions rows"
        )));
    }

    validate_submission_ids(&expected_ids, &submitted).map_err(bad_request)?;

    for answer in &submitted {
        let selected_option_key = match &answer.answer.value {
            AnswerValue::Selected(key) => Some(key.as_str()),
            _ => answer
                .answer
                .selected_option
                .as_ref()
                .map(|opt| opt.key.as_str()),
        };

        if let Some(key) = selected_option_key
            && let Some(options) = options_by_id.get(&answer.question_id)
            && !options.iter().any(|opt| opt.key == key)
        {
            return Err(bad_request(format!(
                "unknown selected option `{key}` for question `{}`",
                answer.question_id
            )));
        }

        let answer_json = serde_json::to_string(&answer.answer.value)
            .map_err(|error| bad_request(format!("invalid answer encoding: {error}")))?;

        tx.execute(
            "UPDATE interview_questions
             SET answer = ?1,
                 selected_option = ?2
             WHERE interview_id = ?3 AND question_id = ?4",
            (
                answer_json.as_str(),
                selected_option_key,
                interview_id.as_str(),
                answer.question_id.as_str(),
            ),
        )
        .map_err(internal)?;
    }

    tx.execute(
        "UPDATE interviews
         SET status = 'answered',
             answered_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
             duration_ms = CAST(
                 (julianday(strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) - julianday(asked_at))
                 * 86400000.0 AS INTEGER
             )
         WHERE interview_id = ?1",
        (&interview_id,),
    )
    .map_err(internal)?;

    tx.commit().map_err(internal)?;

    Ok(Json(SubmitAnswersResponse {
        interview_id,
        accepted: submitted.len(),
        mode: "cross_process_db",
    }))
}

struct DbPendingRow {
    interview_id: String,
    context_type: String,
    context_id: String,
    node_id: Option<String>,
    stage_index: Option<i64>,
    asked_at: String,
    question_id: String,
    position: i64,
    question_text: String,
    header: Option<String>,
    question_type: Option<String>,
    options: Option<String>,
}

fn pipeline_event_to_envelope(event: &PipelineEvent) -> Option<WorkflowInterviewEventEnvelope> {
    match event {
        PipelineEvent::InterviewQuestionAsked {
            interview_id,
            node_id,
            question,
        } => Some(WorkflowInterviewEventEnvelope::QuestionAsked {
            interview_id: interview_id.clone(),
            node_id: node_id.clone(),
            question: question.clone(),
        }),
        PipelineEvent::InterviewAnswerReceived {
            interview_id,
            node_id,
        } => Some(WorkflowInterviewEventEnvelope::AnswerReceived {
            interview_id: interview_id.clone(),
            node_id: node_id.clone(),
        }),
        PipelineEvent::InterviewTimedOut {
            interview_id,
            node_id,
        } => Some(WorkflowInterviewEventEnvelope::TimedOut {
            interview_id: interview_id.clone(),
            node_id: node_id.clone(),
        }),
        _ => None,
    }
}

fn parse_question_type(value: Option<&str>) -> stencila_interviews::interviewer::QuestionType {
    use stencila_interviews::interviewer::QuestionType;

    match value {
        Some("YES_NO") => QuestionType::YesNo,
        Some("MULTIPLE_CHOICE") => QuestionType::MultipleChoice,
        Some("MULTI_SELECT") => QuestionType::MultiSelect,
        Some("CONFIRMATION") => QuestionType::Confirmation,
        Some("FREEFORM") | Some("FREE_TEXT") => QuestionType::Freeform,
        _ => QuestionType::Freeform,
    }
}

fn parse_options(value: Option<&str>) -> Result<Vec<QuestionOption>, serde_json::Error> {
    let Some(options_json) = value else {
        return Ok(Vec::new());
    };

    serde_json::from_str(options_json)
}

fn validate_submission_ids(
    expected_ids: &[String],
    submitted: &[SubmittedAnswer],
) -> Result<(), String> {
    let expected = expected_ids
        .iter()
        .map(String::as_str)
        .collect::<std::collections::HashSet<_>>();

    let mut seen = std::collections::HashSet::new();
    for item in submitted {
        if !expected.contains(item.question_id.as_str()) {
            return Err(format!("unknown question_id `{}`", item.question_id));
        }
        if !seen.insert(item.question_id.as_str()) {
            return Err(format!("duplicate answer for `{}`", item.question_id));
        }
    }

    if seen.len() != expected.len() {
        return Err(format!(
            "partial submission: expected {} answers, got {}",
            expected.len(),
            seen.len()
        ));
    }

    Ok(())
}

async fn open_workspace_db(
    state: &ServerState,
) -> Result<stencila_db::WorkspaceDb, (StatusCode, String)> {
    let stencila_dir = stencila_dirs::closest_stencila_dir(&state.dir, true)
        .await
        .map_err(internal)?;
    let db_path = stencila_dir.join(stencila_dirs::DB_SQLITE_FILE);

    let db = stencila_db::WorkspaceDb::open(&db_path).map_err(internal)?;
    db.migrate(
        "workflows",
        stencila_attractor::sqlite_backend::WORKFLOW_MIGRATIONS,
    )
    .map_err(internal)?;
    db.migrate("interviews", stencila_interviews::INTERVIEW_MIGRATIONS)
        .map_err(internal)?;

    Ok(db)
}

fn bad_request(message: impl Into<String>) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, message.into())
}

fn not_found(message: impl Into<String>) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, message.into())
}

fn conflict(message: impl Into<String>) -> (StatusCode, String) {
    (StatusCode::CONFLICT, message.into())
}

fn internal(error: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("{error}"))
}
