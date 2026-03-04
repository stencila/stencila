---
title: Tools
description: Overview of the tools available to Stencila agents and the guard system that evaluates every tool call.
---

Stencila agents interact with the local environment through a set of tools. Each tool call is evaluated by a **guard system** before execution, providing a friction layer between the agent and potentially risky operations.

## Available Tools

| Tool | Description |
| ---- | ----------- |
| [**Shell**](shell/) | Execute shell commands |
| [**File**](file) | Read, write, edit, and search files |
| [**Web**](web) | Fetch web pages and save content locally |

## Tool Guards

Every tool call passes through a guard before execution. The guard inspects the call parameters — the shell command, file path, or URL — and returns a verdict:

| Verdict   | Effect |
| --------- | ------ |
| **Allow** | The tool call proceeds normally. |
| **Warn**  | The tool call proceeds, but the verdict is logged and the agent is informed of the risk. |
| **Deny**  | The tool call is blocked. The agent receives the reason and a suggestion for an alternative approach. |

Each tool has its own guard with domain-specific rules:

- **[Shell guard](shell/)** — pattern-based command evaluation using safe and destructive regex catalogs
- **[File guard](file#guard-rules)** — path-based risk checks for read, write, edit, patch, and search operations
- **[Web guard](web#guard-rules)** — URL and domain safety checks preventing SSRF, credential exposure, and protocol abuse

### Trust Levels

Every agent session runs at one of three trust levels, which controls how strictly tool calls are guarded:

| Level | Description |
| ----- | ----------- |
| **Low** | Most restrictive. Shell commands default to deny; file and web rules apply strictest verdicts. |
| **Medium** (default) | Default-allow with destructive behavior blocking. |
| **High** | Default-allow with relaxed blocking. Some rules downgrade from Deny to Warn. |

### Audit

All `Warn` and `Deny` verdicts are recorded to the workspace's Stencila database (`<workspace>/.stencila/db.sqlite3`) for post-hoc review. `Allow` verdicts are never recorded.

Each audit event captures:

| Field | Description |
| ----- | ----------- |
| `session_id` | Unique identifier for the agent session |
| `agent_name` | Name of the agent that made the tool call |
| `trust_level` | Trust level of the session (`low`, `medium`, `high`) |
| `tool_name` | Name of the tool that was called (`shell`, `read_file`, `web_fetch`, etc.) |
| `input` | The tool input that was evaluated (command string, file path, URL) |
| `matched_segment` | The specific segment that triggered the rule (normalized path, decisive command segment) |
| `verdict` | The verdict (`Warn` or `Deny`) |
| `rule_id` | The rule that fired (e.g., `shell.core.filesystem.recursive_delete_root`) |
| `reason` | Human-readable explanation of why the rule fired |
| `suggestion` | Actionable suggestion for what to do instead |

Auditing is **best-effort** and **non-blocking**: events are sent through a bounded channel to a background task. If the database cannot be opened or the channel is full, events are silently dropped — guard enforcement continues regardless.
