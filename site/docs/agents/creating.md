---
title: Creating Agents
description: How to create and configure custom agents with AGENT.md files.
---

## Quick Start

Create a new agent with the CLI:

```sh
stencila agents create my-agent "A helpful coding assistant"
```

This creates `.stencila/agents/my-agent/AGENT.md` in your workspace with a template you can edit. To create a user-level agent (shared across all workspaces), add `--user`:

```sh
stencila agents create my-agent "A helpful coding assistant" --user
```

## The AGENT.md File

An agent is a directory containing an `AGENT.md` file. The file has two parts:

1. **YAML frontmatter** — configuration (name, model, provider, tools, etc.)
2. **Markdown body** (optional) — system instructions appended to the prompt

Frontmatter property names can be written in camelCase, snake_case, or kebab-case. We recommend **kebab-case** for readability.

Here is a minimal example:

```markdown
---
name: code-engineer
description: A general-purpose coding agent
---
```

And a fully configured example:

```markdown
---
name: code-reviewer
description: Reviews code for correctness, style, and security issues
keywords:
  - code
  - review
  - audit
  - security
when-to-use:
  - when the user asks to review, audit, or critique code changes
  - when a pull request needs automated review before merging
when-not-to-use:
  - when the user wants to write or generate new code
  - when the task is about refactoring rather than review
model: claude-sonnet-4-5
provider: anthropic
reasoning-effort: high
trust-level: medium
allowed-tools:
  - read_file
  - grep
  - glob
  - shell
---

You are a code reviewer. When asked to review code:

1. Read the files and understand the change
2. Check for correctness, security issues, and style problems
3. Suggest concrete improvements with code examples
4. Do not modify files — only read and analyze
```

## Agent Names

Agent names must be **lowercase kebab-case**:

- 1–64 characters
- Only lowercase alphanumeric characters and hyphens
- No leading, trailing, or consecutive hyphens

By convention, names follow a `thing-role` pattern describing the agent's domain and function:

| Name | Domain | Role |
| ---- | ------ | ---- |
| `code-engineer` | code | engineer |
| `code-reviewer` | code | reviewer |
| `data-analyst` | data | analyst |
| `site-designer` | site | designer |

The agent's directory name must match the `name` field in the frontmatter. Validation will flag mismatches.

## Directory Structure

Agent definitions live in an `agents/` directory under `.stencila/` (workspace) or `~/.config/stencila/` (user). Each agent gets its own subdirectory:

```
.stencila/
  agents/
    code-engineer/
      AGENT.md
    code-reviewer/
      AGENT.md
    data-analyst/
      AGENT.md
```

## Choosing Models and Providers

The `models`, `providers`, and `model-size` fields control which LLM the agent uses. All are optional:

```yaml
models:
  - claude-sonnet-4-5
  - gpt-5.2-codex
providers:
  - anthropic
  - openai
model-size: medium
```

The singular `model` and `provider` keys still work for backward compatibility:

```yaml
model: claude-sonnet-4-5
provider: anthropic
```

- Routing precedence is `models` > `model-size` > `providers` > defaults.
- If only `models` or `model` is set, the provider is inferred from each model name.
- If only `providers` or `provider` is set, the default model for the first available provider is used.
- If `model-size` is set, Stencila selects the best available model in that size tier, optionally constrained by `providers`.
- If neither is set, the first available provider with valid credentials is used.

Use `model-size` when you want to express a broad tradeoff such as “use a small, fast, cheap model” without hard-coding a specific model ID. Stencila treats model size as a cross-provider classification, grouping provider models into broad tiers such as `small`, `medium`, and `large`.

These tiers are a Stencila abstraction, not a provider-standard guarantee. A `small` model from one provider is not expected to be exactly equivalent to a `small` model from another provider; the classification is intended to normalize rough cost, latency, and capability tradeoffs across providers.

If you combine `model-size` with `providers`, the provider list constrains which providers Stencila can choose from, and `model-size` selects the preferred tier within those providers.

For example:

- Use `model-size: small` for simple routing, formatting, summarization, or triage agents.
- Use `model-size: medium` for general-purpose coding, analysis, or research agents.
- Use `model-size: large` for difficult planning, deep review, or high-stakes reasoning tasks.

Supported providers: `anthropic`, `openai`, `gemini` (or `google`), `mistral`, `deepseek`.

For CLI-backed sessions, set the provider to the CLI variant: `claude-cli`, `codex-cli`, or `gemini-cli`.

## Improving Discoverability and Delegation

When a manager agent decides which agent to delegate to, it uses the agent's `description`, `keywords`, `when-to-use`, and `when-not-to-use` fields to make its choice. Adding these fields improves delegation accuracy.

### Keywords

The `keywords` field helps managers find and rank agents. Include terms that reflect the agent's domain, capabilities, and likely user intents:

```yaml
keywords:
  - code
  - review
  - security
  - audit
```

### When to Use / When Not to Use

The `when-to-use` and `when-not-to-use` fields provide explicit selection signals that help managers choose between agents with overlapping descriptions:

```yaml
when-to-use:
  - when the user asks to review or audit code
  - when a pull request needs automated review
when-not-to-use:
  - when the user wants to write or generate new code
  - when the task is refactoring rather than review
```

Each entry should be a short, specific sentence describing a scenario. Avoid vague signals like "when appropriate" — be concrete about the situations that match or don't match.

## Restricting Tools

By default, agents have access to all tools registered for their provider. Use `allowed-tools` to restrict an agent to specific tools:

```yaml
allowed-tools:
  - read_file
  - grep
  - glob
```

This is useful for agents that should only read (not write) files, or agents that should not have shell access.

## Building Single-Skill Agents

If you want an agent to focus on one workflow only, set `allowed-skills` to a single skill name:

```yaml
allowed-skills:
  - code-review
```

When an agent has exactly one allowed skill, Stencila automatically preloads that skill's full instructions into the initial system prompt in addition to making it available via `use_skill`. This avoids spending an extra model turn to load the only permitted skill and makes single-skill agents more reliable.

If you list multiple skills, only their metadata is included initially and the model can load the full content of a skill on demand with `use_skill`.

## Trust Levels

The `trust-level` field controls how strictly the agent's tool calls are guarded:

| Level | Behavior |
| ----- | -------- |
| `low` | Shell commands default to deny unless they match a known-safe pattern. Strictest file and web rules. |
| `medium` (default) | Default-allow with destructive behavior blocking. |
| `high` | Default-allow with relaxed blocking. |

```yaml
trust-level: low
```

## Reasoning Effort

Control how much the model reasons before responding with the `reasoning-effort` field:

```yaml
reasoning-effort: high
```

Valid values: `low`, `medium`, `high`. When not set, the provider's default is used. Higher reasoning effort uses more tokens but can improve quality on complex tasks.

`reasoning-effort` and `model-size` control different things:

- `model-size` chooses which class of model to use.
- `reasoning-effort` controls how much deliberation that chosen model applies before responding.

In other words, `model-size` is about model capability tier and cost/latency tradeoffs, while `reasoning-effort` is about how hard the selected model should think.

## Limiting Turns and Tool Rounds

Control how long an agent can run:

```yaml
max-turns: 20           # Maximum conversation turns (0 = unlimited)
max-tool-rounds: 10    # Maximum tool-call rounds per user input
tool-timeout: 60       # Default tool timeout in seconds
max-subagent-depth: 2  # Maximum subagent nesting depth
```

## MCP Server Integration

Agents can use tools from [MCP (Model Context Protocol)](https://modelcontextprotocol.io/) servers. Two modes are available:

**Codemode** (default, token-efficient): A single `mcp_codemode` tool lets the model write JavaScript to orchestrate MCP calls in a sandboxed environment.

```yaml
enable-mcp-codemode: true  # default
```

**Direct registration**: Each MCP server tool is registered individually. Simpler but uses more tokens.

```yaml
enable-mcp: true
enable-mcp-codemode: false
```

Restrict which MCP servers an agent can access:

```yaml
allowed-mcp-servers:
  - context7
  - my-database
```

## Domain Restrictions

Control which domains the agent can access with `web_fetch`:

```yaml
# Allow only specific domains
allowed-domains:
  - docs.rs
  - "*.github.com"

# Or deny specific domains
disallowed-domains:
  - internal.corp.example.com
```

Wildcard subdomains (`*.example.com`) are supported. When both lists are set, the allowlist takes precedence.

## Validation

Validate an agent definition before using it:

```sh
# Validate by name
stencila agents validate code-reviewer

# Validate by path
stencila agents validate .stencila/agents/code-reviewer/

# Validate an AGENT.md file directly
stencila agents validate .stencila/agents/code-reviewer/AGENT.md
```

Validation checks for **errors** (the agent cannot be used):

- Name format (kebab-case, 1–64 characters)
- Name matches directory name
- Description is non-empty and not a placeholder
- Numeric fields are within valid ranges
- Compatibility string length (max 500 characters) — see [Configuration Reference](configuration#compatibility)

Validation also checks for **warnings** (advisory, the agent can still be used):

- **Skill tool coverage** — when the agent has an `allowed-tools` list, the validator cross-references it against skills' `allowed-tools` declarations. If a skill needs a tool that the agent doesn't allow, a warning is shown. This helps catch configuration mismatches where a skill would be unable to use a tool it expects.

## Configuration-Only Agents

An agent does not need a Markdown body. A frontmatter-only `AGENT.md` is valid and simply configures which model, provider, and settings to use without adding custom system instructions:

```markdown
---
name: fast-coder
description: Quick coding tasks with a fast model
model: claude-haiku-3-5
provider: anthropic
reasoning-effort: low
max-turns: 5
---
```

This is useful for creating model/provider shortcuts or for agents where the project docs (`AGENTS.md`, `CLAUDE.md`, etc.) already provide sufficient instructions.
