-- Initial schema for agent tool guard audit events.

CREATE TABLE IF NOT EXISTS agent_tool_guard_events (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    session_id       TEXT NOT NULL,
    agent_name       TEXT NOT NULL,
    trust_level      TEXT NOT NULL,
    tool_name        TEXT NOT NULL,
    input            TEXT NOT NULL,
    matched_segment  TEXT NOT NULL,
    verdict          TEXT NOT NULL CHECK(verdict IN ('Warn', 'Deny')),
    rule_id          TEXT NOT NULL,
    reason           TEXT,
    suggestion       TEXT
);
CREATE INDEX IF NOT EXISTS idx_atg_ts      ON agent_tool_guard_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_atg_session ON agent_tool_guard_events(session_id);
CREATE INDEX IF NOT EXISTS idx_atg_agent   ON agent_tool_guard_events(agent_name);
CREATE INDEX IF NOT EXISTS idx_atg_verdict ON agent_tool_guard_events(verdict);
CREATE INDEX IF NOT EXISTS idx_atg_tool    ON agent_tool_guard_events(tool_name);
CREATE INDEX IF NOT EXISTS idx_atg_rule    ON agent_tool_guard_events(rule_id);
