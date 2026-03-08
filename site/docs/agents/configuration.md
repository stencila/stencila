---
title: Configuration Reference
description: Full reference for AGENT.md frontmatter properties.
---

This page documents all properties available in the YAML frontmatter of an `AGENT.md` file.

## Required Properties

### `name`

**Type:** `string` — **Required**

The name of the agent. Must be lowercase kebab-case: 1–64 characters, only lowercase alphanumeric characters and hyphens, no leading/trailing/consecutive hyphens.

```yaml
name: code-engineer
```

### `description`

**Type:** `string` — **Required**

A brief description of the agent. Must be non-empty and at most 1024 characters.

```yaml
description: A general-purpose coding agent for software engineering tasks
```

## Model and Provider

### `model`

**Type:** `string`

Model identifier, e.g. `claude-sonnet-4-5`, `gpt-5.2-codex`. When not specified, the default model for the provider is used. Model aliases (e.g. `claude`, `gpt`, `gemini`) resolve to the latest model for that provider.

```yaml
model: claude-sonnet-4-5
```

### `provider`

**Type:** `string`

Provider identifier: `anthropic`, `openai`, `gemini` (or `google`), `mistral`, `deepseek`. For CLI-backed sessions, use the CLI variant: `claude-cli`, `codex-cli`, `gemini-cli`. When not specified, the provider is inferred from the model name or the first available provider is used.

```yaml
provider: anthropic
```

### `reasoningEffort`

**Type:** `string`

Reasoning effort level: `low`, `medium`, `high`, or a custom provider-specific value. Controls how much the model reasons before responding. Higher effort uses more tokens but can improve quality. When not specified, the provider's default is used.

```yaml
reasoningEffort: high
```

## Safety and Access Control

### `trustLevel`

**Type:** `string`

Trust level controlling how strictly the agent's tool calls are guarded. See [Tool Guards](tools/#tool-guards) for details.

| Value | Description |
| ----- | ----------- |
| `low` | Shell is default-deny; strictest file and web rules |
| `medium` | Default-allow with destructive behavior blocking (default) |
| `high` | Default-allow with relaxed blocking |

```yaml
trustLevel: low
```

### `allowedTools`

**Type:** `string[]`

Tool names this agent is allowed to use. When set, only the listed tools are sent to the model and allowed to execute. When unset, all tools registered for the provider are available.

```yaml
allowedTools:
  - read_file
  - write_file
  - edit_file
  - grep
  - glob
  - shell
  - web_fetch
```

When validating an agent (via `stencila agents validate`), the validator cross-references this list against the `allowedTools` declared by the agent's skills. If a skill needs a tool not in the agent's `allowedTools`, a warning is shown. See [Creating Agents — Validation](creating#validation) for details.

### `allowedDomains`

**Type:** `string[]`

Domain allowlist for `web_fetch`. Supports exact hosts and `*.` wildcard subdomain entries. When set, domains not in this list are denied.

```yaml
allowedDomains:
  - docs.rs
  - "*.github.com"
  - crates.io
```

### `disallowedDomains`

**Type:** `string[]`

Domain denylist for `web_fetch`. Supports exact hosts and `*.` wildcard subdomain entries. When both `allowedDomains` and `disallowedDomains` are set, the allowlist takes precedence.

```yaml
disallowedDomains:
  - internal.corp.example.com
```

## Skills

### `allowedSkills`

**Type:** `string[]`

Skill names this agent can use. When unset, all discovered skills are available. When set to a non-empty array, only the listed skills are available. When set to an empty array, skills are disabled entirely. See the [skills documentation](../skills/) for more on creating and using skills.

```yaml
allowedSkills:
  - testing
  - documentation
```

## MCP Integration

### `enableMcp`

**Type:** `boolean` — Default: `false`

Whether to register MCP server tools directly in the agent's tool registry. Each tool from every connected MCP server is registered individually. This is simple but token-expensive — prefer `enableMcpCodemode` for most agents.

```yaml
enableMcp: true
```

### `enableMcpCodemode`

**Type:** `boolean` — Default: `true`

Whether to register a single `mcp_codemode` tool for MCP orchestration. The model writes JavaScript to orchestrate MCP calls in a sandboxed environment. TypeScript declarations are included in the system prompt. Much more token-efficient than direct MCP tool registration.

```yaml
enableMcpCodemode: true
```

### `allowedMcpServers`

**Type:** `string[]`

MCP server IDs this agent is allowed to use. When unset, all discovered and connected MCP servers are available. When set, only the listed server IDs are used.

```yaml
allowedMcpServers:
  - context7
  - my-database
```

## Session Limits

### `maxTurns`

**Type:** `integer` — Default: `0` (unlimited)

Maximum total conversation turns. When reached, the session ends.

```yaml
maxTurns: 20
```

### `maxToolRounds`

**Type:** `integer`

Maximum tool-call rounds per user input. Limits how many times the model can call tools before it must respond to the user.

```yaml
maxToolRounds: 10
```

### `toolTimeout`

**Type:** `integer`

Default timeout for tool execution, in seconds. Must be greater than 0.

```yaml
toolTimeout: 60
```

### `maxSubagentDepth`

**Type:** `integer` — Default: `1`

Maximum nesting depth for subagents. Controls how many levels deep subagents can spawn their own subagents.

```yaml
maxSubagentDepth: 2
```

## Context Management

### `historyThinkingReplay`

**Type:** `string` — Default: `none`

Controls whether chain-of-thought content is included when replaying assistant turns in subsequent requests.

| Value | Description |
| ----- | ----------- |
| `none` | Strip all thinking and reasoning from history, saving context space (default) |
| `full` | Replay thinking and reasoning content as-is |

```yaml
historyThinkingReplay: full
```

### `truncationPreset`

**Type:** `string` — Default: `balanced`

Named preset for tool output truncation limits. Controls how aggressively tool outputs are truncated before being included in conversation context.

| Value | Description |
| ----- | ----------- |
| `strict` | Tighter limits, preserves more context budget for conversation |
| `balanced` | Moderate limits suitable for most agents (default) |
| `verbose` | No additional truncation beyond spec defaults |

```yaml
truncationPreset: strict
```

### `compactionTriggerPercent`

**Type:** `integer` — Default: `70`

Context usage percentage that triggers proactive history compaction. When estimated context usage exceeds this percentage of the model's context window, the agent proactively compacts conversation history. Set to `0` to disable proactive compaction.

```yaml
compactionTriggerPercent: 80
```

## Metadata

### `compatibility`

**Type:** `string`

Environment requirements for the agent. Max 500 characters. Indicates intended product, required system packages, network access needs, etc. Most agents do not need this field.

```yaml
compatibility: Requires Python 3.11+ and access to a PostgreSQL database
```

## Markdown Body (Instructions)

The Markdown content after the frontmatter closing `---` is the agent's **system instructions**. These are appended to the system prompt as the highest-precedence user instruction layer.

```markdown
---
name: code-reviewer
description: Reviews code for issues
---

You are a code reviewer. Follow these principles:

- Focus on correctness and security
- Suggest improvements with concrete code examples
- Be concise — flag only meaningful issues
- Do not modify files, only read and analyze
```

The body is optional. An agent with only frontmatter is valid and useful for model/provider configuration without custom instructions.
