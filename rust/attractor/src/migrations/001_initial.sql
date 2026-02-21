-- Initial schema for workflow SQLite backend.

-- Content-addressed, deduplicated snapshots of definition directories.
-- kind: 'workflow', 'agent', 'skill'
-- content: zstd-compressed tar archive of the entire definition folder
--          (e.g. the workflow/agent/skill directory including the main
--          markdown file and any supporting files).
CREATE TABLE IF NOT EXISTS workflow_definition_snapshots (
    content_hash TEXT PRIMARY KEY,
    content     BLOB NOT NULL,
    kind        TEXT NOT NULL,
    name        TEXT NOT NULL,
    first_seen_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- One row per workflow execution.
CREATE TABLE IF NOT EXISTS workflow_runs (
    run_id          TEXT PRIMARY KEY,
    workflow_name   TEXT NOT NULL,
    stencila_version TEXT NOT NULL DEFAULT '',
    goal            TEXT NOT NULL DEFAULT '',
    started_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    completed_at    TEXT,
    status          TEXT NOT NULL DEFAULT 'running',
    total_duration_ms INTEGER,
    total_tokens    INTEGER DEFAULT 0,
    node_count      INTEGER DEFAULT 0
);

-- Join table: run → definition snapshots that contributed to it.
CREATE TABLE IF NOT EXISTS workflow_run_definitions (
    run_id       TEXT NOT NULL REFERENCES workflow_runs(run_id),
    content_hash TEXT NOT NULL REFERENCES workflow_definition_snapshots(content_hash),
    role         TEXT NOT NULL,
    PRIMARY KEY (run_id, content_hash, role)
);

-- Key-value context store scoped per run.
CREATE TABLE IF NOT EXISTS workflow_context (
    run_id     TEXT NOT NULL REFERENCES workflow_runs(run_id),
    key        TEXT NOT NULL,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (run_id, key)
);

-- Per-node execution metrics.
CREATE TABLE IF NOT EXISTS workflow_nodes (
    run_id         TEXT NOT NULL REFERENCES workflow_runs(run_id),
    node_id        TEXT NOT NULL,
    status         TEXT NOT NULL DEFAULT 'pending',
    started_at     TEXT,
    completed_at   TEXT,
    duration_ms    INTEGER,
    model          TEXT,
    provider       TEXT,
    input_tokens   INTEGER DEFAULT 0,
    output_tokens  INTEGER DEFAULT 0,
    retry_count    INTEGER DEFAULT 0,
    failure_reason TEXT,
    PRIMARY KEY (run_id, node_id)
);

-- Per-node LLM responses.
CREATE TABLE IF NOT EXISTS workflow_node_responses (
    run_id   TEXT NOT NULL REFERENCES workflow_runs(run_id),
    node_id  TEXT NOT NULL,
    response BLOB NOT NULL,
    PRIMARY KEY (run_id, node_id)
);

-- Edge traversal history.
CREATE TABLE IF NOT EXISTS workflow_edges (
    run_id     TEXT NOT NULL REFERENCES workflow_runs(run_id),
    step_index INTEGER NOT NULL,
    from_node  TEXT NOT NULL,
    to_node    TEXT NOT NULL,
    edge_label TEXT,
    timestamp  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (run_id, step_index)
);

-- Artifact registry (filesystem paths, not content).
CREATE TABLE IF NOT EXISTS workflow_artifacts (
    run_id      TEXT NOT NULL REFERENCES workflow_runs(run_id),
    artifact_id TEXT NOT NULL,
    name        TEXT NOT NULL,
    mime_type   TEXT,
    size_bytes  INTEGER,
    path        TEXT NOT NULL,
    stored_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (run_id, artifact_id)
);

-- Human-in-the-loop interaction records.
CREATE TABLE IF NOT EXISTS workflow_interviews (
    interview_id    TEXT PRIMARY KEY,
    run_id          TEXT NOT NULL REFERENCES workflow_runs(run_id),
    node_id         TEXT NOT NULL,
    question_text   TEXT NOT NULL,
    question_type   TEXT,
    options         TEXT,
    answer          TEXT,
    selected_option TEXT,
    asked_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    answered_at     TEXT,
    duration_ms     INTEGER
);
CREATE INDEX IF NOT EXISTS idx_workflow_interviews_run ON workflow_interviews(run_id);

-- Append-only run log.
CREATE TABLE IF NOT EXISTS workflow_logs (
    run_id    TEXT NOT NULL REFERENCES workflow_runs(run_id),
    timestamp TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    entry     TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_workflow_logs_run ON workflow_logs(run_id);
