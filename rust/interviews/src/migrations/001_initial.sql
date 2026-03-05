-- Interview persistence schema.
--
-- Records human-in-the-loop interactions from both workflow pipeline gates
-- and standalone agent sessions. Context-agnostic: the `context_type` +
-- `context_id` pair identifies the owning context.
--
-- Interviews are the parent entity (one per interaction), and
-- interview_questions are the child entities (one per question within
-- an interview). Single-question interviews have one child row.

-- Interview-level record (one per interaction)
CREATE TABLE IF NOT EXISTS interviews (
    interview_id    TEXT PRIMARY KEY,
    context_type    TEXT NOT NULL,  -- 'workflow' | 'agent_session'
    context_id      TEXT NOT NULL,  -- run_id for workflows, session_id for agent sessions
    node_id         TEXT,           -- NULL for standalone agent sessions
    stage_index     INTEGER,       -- disambiguates multiple visits to the same node (loops/retries)
    status          TEXT NOT NULL DEFAULT 'pending',  -- 'pending' | 'answered' | 'timeout' | 'skipped' | 'error'
    asked_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    answered_at     TEXT,
    duration_ms     INTEGER,
    metadata        TEXT           -- JSON object for interview-level metadata
);
CREATE INDEX IF NOT EXISTS idx_interviews_context ON interviews(context_type, context_id);
CREATE INDEX IF NOT EXISTS idx_interviews_pending ON interviews(context_type, context_id, node_id, status);

-- Individual questions within an interview
CREATE TABLE IF NOT EXISTS interview_questions (
    question_id     TEXT PRIMARY KEY,
    interview_id    TEXT NOT NULL REFERENCES interviews(interview_id) ON DELETE CASCADE,
    position        INTEGER NOT NULL,
    question_text   TEXT NOT NULL,
    header          TEXT,
    question_type   TEXT,
    options         TEXT,          -- JSON array of QuestionOption
    answer          TEXT,          -- JSON-serialized AnswerValue
    selected_option TEXT,
    UNIQUE(interview_id, position)
);
