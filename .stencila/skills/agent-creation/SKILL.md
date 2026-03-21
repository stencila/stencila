---
name: agent-creation
description: Create a new Stencila agent. Use when asked to create, write, scaffold, or set up an agent directory or AGENT.md file. Covers workspace and user-level agents with model, provider, tool, trust, and MCP configuration.
keywords:
  - agent
  - create
  - scaffold
  - write
  - set up
  - AGENT.md
allowed-tools: read_file write_file edit_file apply_patch glob grep shell ask_user
---

## Overview

Create a new agent directory and `AGENT.md` file for Stencila. An agent is a directory containing an `AGENT.md` file with YAML frontmatter (configuration) and an optional Markdown body (system instructions). Agents live under `.stencila/agents/` (workspace) or `~/.config/stencila/agents/` (user).

## Steps

1. Determine the agent name and description from the user's request
2. Validate the name against the naming rules below
3. Decide the target location:
   - **Workspace agent** (default): resolve the closest workspace by walking up from the current directory to find the nearest `.stencila/` directory. If none exists, create `.stencila/agents/<name>/` at the repository root (or the current working directory if not in a repository). Create the agent under `<workspace>/.stencila/agents/<name>/`
   - **User agent** (when the user says "user-level", "global", or "shared across workspaces"): create under `~/.config/stencila/agents/<name>/`
4. Ask the user about model/provider preferences if not specified, or use the defaults from the Choosing Configuration section below
5. Add `keywords`, `when-to-use`, and `when-not-to-use` to help managers select this agent — include terms from the user's request and relevant domain words in `keywords`, and add `when-to-use`/`when-not-to-use` signals describing when this agent should or should not be chosen
6. Write the `AGENT.md` file with frontmatter and optional system instructions
7. Replace all placeholders such as `TODO` before considering the agent complete
8. **Top-down design**: When the user wants to design the agent first and create its skills afterward:
   - list planned skills in `allowed-skills` using kebab-case names that follow naming conventions, even if no corresponding `SKILL.md` exists yet
   - note which skills need to be created and inform the user of the outstanding dependencies
   - validation and the runtime accept forward references (unresolved skills produce a runtime warning, not an error), so the agent can be authored and iterated on before all skill dependencies are in place
9. Validate the finished agent with `stencila agents validate <name>`, the agent directory path, or the `AGENT.md` path

## Naming Rules

Agent names must be **lowercase kebab-case**:

- 1–64 characters
- Only lowercase alphanumeric characters and hyphens (`a-z`, `0-9`, `-`)
- Must not start or end with a hyphen
- Must not contain consecutive hyphens (`--`)
- Must match the parent directory name
- Pattern: `^[a-z0-9]([a-z0-9-]{0,62}[a-z0-9])?$`

By convention, names follow a `thing-role` pattern describing the agent's domain and function (e.g., `code-engineer`, `code-reviewer`, `data-analyst`, `site-designer`).

Common corrections: `codeReviewer` → `code-reviewer`, `data_analyst` → `data-analyst`, `Code-Engineer` → `code-engineer`.

## AGENT.md Format

The file has two parts:

1. **YAML frontmatter** between `---` delimiters — configuration
2. **Markdown body** (optional) — system instructions appended to the agent's prompt

Use kebab-case for all frontmatter property names.

### Required frontmatter fields

- `name` — the agent name (must match directory name)
- `description` — what the agent does (max 1,024 characters)

### Optional frontmatter fields

These fields correspond to properties in the `Agent` schema (`schema/Agent.yaml`).

- `model` — model identifier. When omitted, the provider's default model is used.
- `provider` — provider identifier. For CLI-backed sessions, use the CLI variant (e.g., `claude-cli`). When omitted, inferred from the model name or first available provider.
- `model-size` — preferred model size tier, e.g. `small`, `medium`, `large`. Use this to express broad cost/latency/capability preferences without hard-coding a model.
- `reasoning-effort` — `low`, `medium`, or `high`. Controls model reasoning depth.
- `trust-level` — `low`, `medium` (default), or `high`. Controls tool call guard strictness.
- `allowed-tools` — list of tool names the agent may use. Use a YAML list (one item per line) to avoid confusion. When omitted, all provider tools are available.
- `allowed-skills` — list of skill names the agent may use. When omitted, all discovered skills are available. Set to `[]` to disable skills. When set to exactly one skill, that skill's full content is automatically preloaded into the system prompt in addition to being available via `use_skill`. Forward references to skills that do not yet exist are valid — see step 8 (Top-down design).
- `allowed-domains` — domain allowlist for `web_fetch`. Supports `*.example.com` wildcards.
- `disallowed-domains` — domain denylist for `web_fetch`.
- `max-turns` — maximum conversation turns (0 = unlimited, default: 0).
- `max-tool-rounds` — maximum tool-call rounds per user input.
- `tool-timeout` — default tool timeout in seconds.
- `max-subagent-depth` — maximum subagent nesting depth (default: 1).
- `enable-mcp` — register MCP tools directly (default: false).
- `enable-mcp-codemode` — register the `mcp_codemode` orchestration tool (default: true).
- `allowed-mcp-servers` — list of MCP server IDs the agent may use.
- `history-thinking-replay` — `none` (default) or `full`. Controls chain-of-thought replay in history.
- `truncation-preset` — `strict`, `balanced` (default), or `verbose`. Controls tool output truncation.
- `compaction-trigger-percent` — context usage percentage that triggers history compaction (default: 70).
- `compatibility` — environment requirements (max 500 characters).
- `keywords` — list of keywords or tags for discovery and routing; use terms that reflect likely user intents, artifacts, and domains
- `when-to-use` (positive selection signals) and `when-not-to-use` (negative selection signals); help managers choose the right resource

If an agent does not need tool restrictions, prefer omitting `allowed-tools`. If an agent does set `allowed-tools` and it also needs one or more skills, ensure it includes all tools allowed by each allowed skill so the agent can actually execute those skills without tool-coverage mismatches. Include `use_skill` when the agent may need to invoke skills dynamically; for exactly one allowed skill, Stencila preloads that skill into the prompt so `use_skill` is not required just for that preloaded skill.

## Common Agent Patterns

### Minimal agent (configuration only)

```markdown
---
name: fast-coder
description: Quick coding tasks with a fast model
max-turns: 5
---
```

### Read-only agent (restricted tools)

```markdown
---
name: code-reviewer
description: Reviews code for correctness, style, and security issues
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

### Single-skill agent

When an agent should follow exactly one skill, set `allowed-skills` to that skill name. Stencila preloads the skill's full instructions into the system prompt automatically, so the model doesn't spend a turn calling `use_skill`.

Add a short Markdown body (one or two sentences) that frames the agent's identity and specialization. The preloaded skill instructions are appended after this preamble, so the body should set context rather than repeat the skill.

If you include `allowed-tools` on a skill-using agent, include all tools allowed by that skill. Add `use_skill` only if the agent may need to invoke skills dynamically; for exactly one allowed skill, the preloaded skill does not require `use_skill`.

```markdown
---
name: code-reviewer
description: Reviews code using the code-review skill
allowed-skills:
  - code-review
allowed-tools:
  - read_file
  - grep
  - glob
  - shell
---

You are an assistant that specializes in reviewing code for correctness, style, and security issues.
```

### Full-featured agent

```markdown
---
name: code-engineer
description: A general-purpose coding agent for software engineering tasks
keywords:
  - code
  - implement
  - debug
  - refactor
when-to-use:
  - when the user asks to write, implement, debug, or refactor code
  - when the task involves software engineering work
when-not-to-use:
  - when the user only wants a code review without modifications
trust-level: medium
allowed-tools:
  - read_file
  - write_file
  - edit_file
  - grep
  - glob
  - shell
  - web_fetch
max-turns: 0
max-tool-rounds: 25
tool-timeout: 120
---

You are a software engineer. Follow these principles:

- Write clean, readable code that follows the project's existing conventions
- Prefer simple, focused changes over large refactors
- Handle errors appropriately
- Do not introduce security vulnerabilities
```

## Example Walkthrough

Input: "Create an agent for reviewing pull requests that only reads files"

Process:

1. Derive name: `pr-reviewer` (thing-role pattern, kebab-case)
2. Resolve workspace: find nearest `.stencila/` directory, e.g., at the repository root
3. Target path: `.stencila/agents/pr-reviewer/AGENT.md`
4. Select read-only tools, high reasoning effort, no model/provider (use defaults)
5. Write the file, then validate

Output:

```markdown
---
name: pr-reviewer
description: Reviews pull requests for correctness, security, and style issues
reasoning-effort: high
allowed-tools:
  - read_file
  - grep
  - glob
  - shell
---

You are a pull request reviewer. For each review:

1. Read the changed files and understand the intent
2. Check for bugs, security issues, and style problems
3. Provide specific, actionable feedback with code examples
4. Do not modify any files
```

Validated with: `stencila agents validate pr-reviewer`

## Choosing Configuration

Do not hard-code specific model names or providers in agent definitions unless the user explicitly requests one. Models and providers change frequently, and users may not have API keys for a given provider. Omitting `model` and `provider` lets Stencila resolve the best available option at runtime. Prefer `model-size` when the user expresses a general preference such as speed, cost, or capability tier without naming a specific model. Do not guess or invent model/provider identifiers — use only values the user explicitly provides, or omit the fields entirely. Invalid identifiers are not caught by validation and will fail at runtime.

Treat `model-size` as a cross-provider size classification managed by Stencila. It is useful for agents with simple or repetitive tasks that should prefer smaller, faster, cheaper models. These tiers are approximate across providers, not exact equivalence classes.

Keep `model-size` and `reasoning-effort` conceptually separate:

- `model-size` chooses the class of model to use.
- `reasoning-effort` controls how much deliberation that chosen model applies.

If the user specifies both `model-size` and `providers`, prefer that combination over hard-coding `model` values unless they explicitly ask for exact models.

Guide the user with these defaults when they don't specify preferences:

| Use case | Model size | Trust | Reasoning | Key tools |
| -------- | ---------- | ----- | --------- | --------- |
| General coding | `medium` | `medium` | `medium` | all (omit `allowed-tools`) |
| Code review | `medium` or `large` | `low` or `medium` | `high` | `read_file`, `grep`, `glob`, `shell` |
| Quick tasks | `small` | `medium` | `low` | all |
| Data analysis | `medium` or `large` | `medium` | `high` | all |
| Documentation | `small` or `medium` | `low` | `medium` | `read_file`, `write_file`, `edit_file`, `grep`, `glob` |

When in doubt, omit optional fields — Stencila uses sensible defaults. If the user clearly wants a faster or cheaper agent, `model-size: small` is often a good fit. If they want stronger analysis or review quality without naming a provider, `model-size: medium` or `large` may be appropriate.

When combining `allowed-skills` with `allowed-tools`, use this rule:

- No tool restrictions needed: omit `allowed-tools`
- Tool restrictions needed and no skills: include only the tools the agent itself needs
- Tool restrictions needed and exactly one allowed skill: include the tools required by that skill, plus any extra agent-specific tools; `use_skill` is optional because the skill is preloaded
- Tool restrictions needed and multiple allowed skills or dynamic skill invocation: include `use_skill` and the union of tools required by those skills, plus any extra agent-specific tools

## Edge Cases

- **Agent directory already exists**: Ask the user whether to overwrite, merge, or abort before modifying an existing agent. Never silently overwrite.
- **Name mismatch**: If the user provides a name that doesn't match kebab-case rules, suggest a corrected version rather than failing silently.
- **Nested workspaces**: If multiple `.stencila/` directories exist in the ancestor chain, use the nearest one. Do not create a duplicate `.stencila/agents/` tree.
- **Empty or placeholder content**: Do not consider the agent complete if any `TODO`, `<placeholder>`, or empty `description` remains in the final `AGENT.md`.
- **User vs workspace confusion**: Confirm with the user if the intent is ambiguous. Default to workspace-level agents.
- **CLI-backed agents**: When the user wants to use a CLI tool, set the provider to the corresponding CLI variant (e.g., `claude-cli`, `codex-cli`, `gemini-cli`).
- **Body is optional**: A frontmatter-only `AGENT.md` is valid. Only add a Markdown body when the user provides custom instructions or the agent needs behavioral guidance beyond what project docs supply.
- **Skill/tool mismatch**: If an agent sets both `allowed-skills` and `allowed-tools`, check that every tool allowed by the selected skills is also included in the agent's `allowed-tools`. Also check whether `use_skill` is needed: it is optional for exactly one preloaded skill, but should be included when the agent may need to invoke skills dynamically or choose among multiple skills.
- **Unknown skills**: List outstanding skill dependencies so the user can create them. This is valid in top-down design (see step 8) — validation passes, and the runtime produces a warning for unresolved skill names, not an error. Do not remove skill references just because the targets do not exist yet.

## Validation

Before finishing, validate the agent:

```sh
# By agent name
stencila agents validate <agent-name>

# By directory path
stencila agents validate .stencila/agents/<agent-name>

# By AGENT.md path
stencila agents validate .stencila/agents/<agent-name>/AGENT.md
```

Validation checks for errors (name format, name–directory match, description present and not placeholder, numeric ranges, compatibility length) and warnings (skill tool coverage mismatches). Validation should pass before you report the agent as complete.

## Limitations

- This skill covers agent structure, configuration, and authoring conventions. It does not verify that chosen models, providers, MCP servers, or external services are available at runtime.
- Validation checks structure and known constraints, but some configuration mistakes may still surface only during execution.
