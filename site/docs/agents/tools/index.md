---
title: Tools
description: Overview of the tools available to Stencila agents, how they enable the agentic loop, provider-aligned toolsets, and the guard system that evaluates every tool call.
---

## Why Tools Matter

Tools are what differentiate an agent from a bare LLM conversation. Without tools, a model can only generate text. It cannot read your files, run your tests, or check whether its suggestions actually work. Tools close the loop between reasoning and action.

Stencila agents operate in an **agentic loop**: the model reasons about a task, requests one or more tool calls, Stencila executes them and returns the results, and the model reasons again based on those results. This cycle repeats until the task is complete or a limit is reached. It is this loop, not the model alone, that makes agents capable of multi-step coding and research tasks like debugging across files, running experiments, or iterating on analysis code until tests pass.

## Provider-Aligned Toolsets

Different model families are trained and optimized for different tool interfaces. OpenAI models produce better edits using their native `apply_patch` format (a v4a diff format). Anthropic models work best with `edit_file`'s `old_string`/`new_string` exact-match replacement. Gemini models expect additional tools like `read_many_files` and `list_dir` that match their reference CLI.

Stencila handles this automatically. When an agent session starts, Stencila selects the tool definitions and system prompts that match the model's provider. Users do not need to configure this; the right tools are provided to the right models.

A core set of tools is shared across all providers. Provider-specific tools extend this base, and optional tools are available to agents that list them in `allowed-tools`:

| Tool | Providers | Description |
| ---- | --------- | ----------- |
| [**Shell**](shell/) | All | Execute shell commands |
| [**File**](file) (`read_file`, `write_file`, `grep`, `glob`) | All | Read, write, and search files |
| [`edit_file`](file#edit_file) | Anthropic, Gemini | Apply exact-match `old_string`/`new_string` replacements |
| [`apply_patch`](file#apply_patch) | OpenAI | Apply multi-file edits in v4a diff format |
| [`read_many_files`](file#read_many_files) | Gemini | Batch-read multiple files in a single call |
| [`list_dir`](file#list_dir) | Gemini | List directory contents with depth options |
| [**Web**](web) (`web_fetch`) | All | Fetch web pages and save content locally |
| [**Snap**](snap) | Opt-in | Capture screenshots and collect structured measurements of served pages |
| [**Orchestration**](orchestration) (`ask_user`, `list_agents`, `list_workflows`, `delegate`) | Opt-in | Discover agents and workflows, delegate tasks, and interact with users |

## Tool Guards

Giving an agent access to tools means giving it the ability to modify files, execute arbitrary commands, and make network requests. A single hallucinated or misguided tool call can cause real damage — deleting files, force-pushing over git history, or leaking credentials into the model's context window.

Tool guards are a **friction layer** that sits between the agent's tool calls and their execution. Every tool call passes through a guard before it runs. The guard inspects the call parameters — the shell command, file path, or URL — and returns a verdict:

| Verdict   | Effect |
| --------- | ------ |
| **Allow** | The tool call proceeds normally. |
| **Warn**  | The tool call proceeds, but the verdict is logged and the agent is informed of the risk. |
| **Deny**  | The tool call is blocked. The agent receives the reason and a suggestion for an alternative approach. |

Each tool has its own guard with domain-specific rules:

- **[Shell guard](shell/)** — pattern-based command evaluation using safe and destructive regex catalogs
- **[File guard](file#guard-rules)** — path-based risk checks for read, write, edit, patch, and search operations
- **[Web guard](web#guard-rules)** — URL and domain safety checks preventing SSRF, credential exposure, and protocol abuse

When a tool call is denied, the agent sees the reason and suggestion as tool output. This means the model can adjust its approach without user intervention (e.g using a safer command, or narrowing a file path).

### Guards and Sandboxes

Tool guards are complementary to OS-level security boundaries such as sandboxes (bubblewrap, Landlock, containers). The two layers address different threat surfaces:

A sandbox constrains what the agent can do to the **local** environment; it cannot write outside a directory, cannot access the network, cannot read sensitive files. But many destructive operations are perfectly valid local commands that cause damage **elsewhere**: `git push --force` destroys history on the remote, `uv publish` pushes a package to PyPI, `aws s3 rm --recursive` deletes cloud storage. These commands can still execute successfully inside sandboxes that allow outbound network access, because the damage happens on a remote system.

Guards address this gap. They distinguish destructive from non-destructive use of tools like `git`, `uv`, `docker`, and cloud CLIs regardless of where the agent runs. A sandboxed agent still needs guards to prevent it from force-pushing a branch or publishing a package by mistake.

### Trust Levels

Rather than requiring users to approve every individual tool call, guards use **trust levels** to control enforcement automatically. Each agent can be configured with a trust level that determines how strictly its tool calls are guarded:

| Level | Description |
| ----- | ----------- |
| **Low** | Most restrictive. Shell commands default to deny unless they match a known-safe pattern. File and web rules apply strictest verdicts. Appropriate for untested agents or sensitive environments. |
| **Medium** (default) | Default-allow with destructive behavior blocking. Known-destructive commands are denied; everything else is allowed. A reasonable default for most use. |
| **High** | Default-allow with relaxed blocking. Some rules that deny at medium trust downgrade to warnings. Appropriate for trusted agents on non-critical workspaces. |

This lets agents work autonomously at a level of freedom appropriate to the context, without requiring a human in the loop for every shell command or file write.

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
