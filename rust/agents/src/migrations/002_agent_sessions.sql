-- Agent session persistence: sessions and turn snapshots.

CREATE TABLE IF NOT EXISTS agent_sessions (
    session_id            TEXT PRIMARY KEY,
    backend_kind          TEXT NOT NULL,
    agent_name            TEXT NOT NULL,
    provider_name         TEXT NOT NULL,
    model_name            TEXT NOT NULL,
    state                 TEXT NOT NULL DEFAULT 'IDLE',
    total_turns           INTEGER NOT NULL DEFAULT 0,
    resumability          TEXT NOT NULL DEFAULT 'Full',
    created_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    workflow_run_id       TEXT,
    workflow_thread_id    TEXT,
    workflow_node_id      TEXT,
    provider_resume_state TEXT,
    config_snapshot       TEXT,
    system_prompt         TEXT,
    lease_holder          TEXT,
    lease_expires_at      TEXT
);

CREATE INDEX IF NOT EXISTS idx_as_agent    ON agent_sessions(agent_name);
CREATE INDEX IF NOT EXISTS idx_as_state    ON agent_sessions(state);
CREATE INDEX IF NOT EXISTS idx_as_updated  ON agent_sessions(updated_at);

CREATE TABLE IF NOT EXISTS agent_session_turns (
    session_id TEXT NOT NULL,
    turn_index INTEGER NOT NULL,
    turn_json  TEXT NOT NULL,
    PRIMARY KEY (session_id, turn_index),
    FOREIGN KEY (session_id) REFERENCES agent_sessions(session_id) ON DELETE CASCADE
);
