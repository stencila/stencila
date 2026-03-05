-- Interview persistence schema.
--
-- Records human-in-the-loop interactions from both workflow pipeline gates
-- and standalone agent sessions. Context-agnostic: the `context_type` +
-- `context_id` pair identifies the owning context.

CREATE TABLE IF NOT EXISTS interviews (
    interview_id    TEXT PRIMARY KEY,
    context_type    TEXT NOT NULL,  -- 'workflow' | 'agent_session'
    context_id      TEXT NOT NULL,  -- run_id for workflows, session_id for agent sessions
    node_id         TEXT,           -- NULL for standalone agent sessions
    question_text   TEXT NOT NULL,
    question_type   TEXT,
    options         TEXT,
    answer          TEXT,
    selected_option TEXT,
    asked_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    answered_at     TEXT,
    duration_ms     INTEGER
);
CREATE INDEX IF NOT EXISTS idx_interviews_context ON interviews(context_type, context_id);
