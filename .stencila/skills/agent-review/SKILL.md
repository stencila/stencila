---
name: agent-review
description: Critically review a Stencila agent and suggest improvements. Use when asked to review, audit, critique, evaluate, or improve an AGENT.md file or agent directory. Covers frontmatter validation, system instruction quality, configuration correctness, and adherence to the Agent schema.
allowed-tools: read_file glob grep shell ask_user
---

## Overview

Review an existing Stencila agent for quality, correctness, and completeness. Produce a structured report with specific, actionable suggestions for improvement. The review covers frontmatter fields, system instruction quality, configuration correctness, security posture, and adherence to the agent schema defined in `schema/Agent.yaml`.

## Steps

1. Identify the agent to review from the user's request — accept an agent name, a directory path, or an `AGENT.md` file path
2. Resolve the agent file: if given a name, look for `.stencila/agents/<name>/AGENT.md` walking up from the current directory; also check `~/.config/stencila/agents/<name>/AGENT.md` for user-level agents. If given a path, use it directly
3. Read the full `AGENT.md` file and any supporting files in the agent directory (check for `scripts/`, `references/`, and `assets/` subdirectories)
4. Read `schema/Agent.yaml` to verify the checklist covers current agent schema fields and to identify any unknown frontmatter properties
5. Evaluate the agent against each criterion in the Review Checklist below
6. Produce a structured review report with a summary, per-criterion findings, and a prioritized list of suggestions
7. If the user asks you to apply the improvements, make the changes and validate the result with `stencila agents validate <agent-name>`

## Review Checklist

### Frontmatter — Required Fields

- **name**: present, matches directory name, valid kebab-case (`^[a-z0-9]([a-z0-9-]{0,62}[a-z0-9])?$`), follows the `thing-role` naming convention (e.g., `code-reviewer`, `data-analyst`)
- **description**: present, not empty, not a placeholder (`TODO`, `<placeholder>`), recommended to be concise (under ~1,024 characters), specific enough to convey the agent's purpose

### Frontmatter — Optional Fields

Check each present field for validity:

- **model**: do not hard-code a model unless the user explicitly requires one; flag hard-coded models as a warning since they reduce portability
- **provider**: same as model — omit unless explicitly needed; flag hard-coded providers as a warning
- **reasoning-effort**: typically `low`, `medium`, or `high` if present; custom provider-specific values are also valid
- **trust-level**: must be `low`, `medium`, or `high` if present; check that it matches the agent's intended use (e.g., a read-only reviewer should not have `high` trust)
- **allowed-tools**: check that listed tools are valid Stencila tool names (`read_file`, `write_file`, `edit_file`, `grep`, `glob`, `shell`, `web_fetch`, `use_skill`, `spawn_agent`, `send_input`, `wait`, `close_agent`, `ask_user`, `mcp_codemode`); flag unknown tool names
- **allowed-skills**: if present, check that listed skill names are valid kebab-case
- **allowed-domains** / **disallowed-domains**: if present, check format (exact hosts or `*.example.com` wildcards)
- **max-turns**: non-negative integer if present
- **max-tool-rounds**: positive integer if present
- **tool-timeout**: positive integer (seconds) if present
- **max-subagent-depth**: non-negative integer if present
- **enable-mcp** / **enable-mcp-codemode**: boolean if present
- **allowed-mcp-servers**: list of server ID strings if present
- **history-thinking-replay**: must be `none` or `full` if present
- **truncation-preset**: must be `strict`, `balanced`, or `verbose` if present
- **compaction-trigger-percent**: unsigned integer (0–100) if present
- **compatibility**: under 500 characters if present
- **unknown fields**: flag any frontmatter fields not defined in `schema/Agent.yaml` (or inherited from `CreativeWork`) as warnings — they may be typos or unsupported properties that will be silently ignored

Note: `Agent` extends `CreativeWork` in the schema, so agents may have additional valid fields such as `license`. Verify against the schema before flagging inherited properties as unknown.

### System Instructions (Markdown Body)

- A missing body is valid — a frontmatter-only `AGENT.md` is a legitimate configuration-only agent. However, a body that exists but contains only empty sections or placeholder content should be flagged
- If present, instructions are clear, imperative, and unambiguous
- Instructions align with the agent's described purpose — a code reviewer's instructions should not describe code generation
- No contradictions between frontmatter configuration and body instructions (e.g., body says "modify files" but `allowed-tools` excludes `write_file` and `edit_file`)
- No placeholder content (`TODO`, `<placeholder>`, or empty sections)

### Security and Least Privilege

- **Tool scope**: agent only has access to tools it needs; flag overly broad tool access for specialized agents (e.g., a documentation agent with `shell` access)
- **Trust level**: appropriate for the agent's role; flag `high` trust on agents that do not need it
- **Domain restrictions**: if the agent uses `web_fetch`, consider whether domain restrictions are appropriate
- **MCP access**: if MCP is enabled, check whether `allowed-mcp-servers` restricts access to only needed servers

### Consistency and Conventions

- Frontmatter property names use kebab-case (not camelCase or snake_case)
- Formatting is consistent (heading levels, list styles, code block languages)
- Naming follows `thing-role` convention
- Configuration choices are internally consistent (e.g., `max-turns: 5` with `reasoning-effort: high` suggests the agent expects complex tasks but has limited turns)

## Report Format

Structure the review as follows:

### Summary

One to three sentences giving an overall assessment and the most important finding.

### Findings

For each checklist area, report one of:

- ✅ **Pass** — criterion fully met
- ⚠️ **Warning** — minor issue or room for improvement
- ❌ **Fail** — significant problem that should be fixed

Include a brief explanation for warnings and failures.

### Suggestions

A numbered list of specific, actionable improvements ordered by priority (most impactful first). Each suggestion should explain *what* to change and *why*.

Use heading level 3 (`###`) for each section in your output.

## Examples

Input: "Review the code-reviewer agent"

Process:

1. Resolve to `.stencila/agents/code-reviewer/AGENT.md`
2. Read the file and check for supporting files in subdirectories (`scripts/`, `references/`, `assets/`)
3. Read `schema/Agent.yaml` to verify field validity
4. Evaluate frontmatter: `name` is `code-reviewer`, matches directory, valid kebab-case, follows `thing-role` convention; `description` is specific
5. Check optional fields: `allowed-tools` lists `read_file`, `grep`, `glob`, `shell` — appropriate for a read-only reviewer
6. Evaluate body: instructions say "do not modify files" — consistent with read-only tools
7. Check security: no `write_file` or `edit_file` — good least privilege
8. Run `stencila agents validate code-reviewer`
9. Produce the report

Output (use `###` headings in the report):

> ### Summary
>
> The code-reviewer agent is well-configured with appropriate read-only tool restrictions and clear instructions. One minor improvement is possible.
>
> ### Findings
>
> | Area | Status | Notes |
> |------|--------|-------|
> | Required fields | ✅ Pass | Name and description are valid and specific |
> | Optional fields | ✅ Pass | All present fields have valid values |
> | System instructions | ✅ Pass | Clear, imperative, consistent with tool restrictions |
> | Security | ✅ Pass | Read-only tools, appropriate trust level |
> | Consistency | ⚠️ Warning | `reasoning-effort` is not set; consider `high` for code review tasks |
>
> ### Suggestions
>
> 1. Add `reasoning-effort: high` — code review benefits from deeper analysis, and this matches the defaults in the agent-creation skill's configuration guide

Input: "Review the quick-helper agent"

Process:

1. Resolve to `.stencila/agents/quick-helper/AGENT.md`
2. Read the file — frontmatter has `name: quick-helper`, `description: Helps with stuff`, `trust-level: high`, `allowed-tools: read_file write_file edit_file shell web_fetch spawn_agent`; body says "You are a helpful assistant" with a `## TODO` section
3. Read `schema/Agent.yaml` to check field validity
4. Evaluate against the checklist
5. Produce the report

Output (use `###` headings in the report):

> ### Summary
>
> The quick-helper agent has significant configuration issues: an overly broad tool set for an unspecified purpose, unnecessarily high trust, a vague description, and placeholder content in the body.
>
> ### Findings
>
> | Area | Status | Notes |
> |------|--------|-------|
> | Required fields | ⚠️ Warning | Description "Helps with stuff" is too vague to convey the agent's purpose |
> | Optional fields | ⚠️ Warning | `trust-level: high` is set without clear justification |
> | System instructions | ❌ Fail | Body contains a `## TODO` placeholder section |
> | Security | ❌ Fail | Agent has `shell`, `web_fetch`, and `spawn_agent` with `trust-level: high` but no clear need for these capabilities |
> | Consistency | ⚠️ Warning | Name `quick-helper` does not follow `thing-role` convention — unclear what domain it covers |
>
> ### Suggestions
>
> 1. Remove placeholder `## TODO` section or replace it with actual instructions
> 2. Reduce `trust-level` to `medium` (or `low`) unless elevated trust is justified by the agent's purpose
> 3. Restrict `allowed-tools` to only the tools the agent needs — remove `shell`, `web_fetch`, and `spawn_agent` unless required
> 4. Rewrite `description` to specifically convey what the agent does and when to use it
> 5. Rename to follow `thing-role` convention (e.g., `general-assistant` or a more specific name)

## Edge Cases

- **Agent not found**: Report the error clearly and suggest checking the name or path. List available agents if possible using `stencila agents list` or by listing `.stencila/agents/` directories.
- **Multiple agents requested**: Review each agent separately with its own report section. Ask the user to confirm if reviewing all agents is intended.
- **Frontmatter-only agent (no body)**: This is valid — do not flag it as a failure. A frontmatter-only `AGENT.md` is a legitimate configuration-only agent.
- **User-level agent**: Check `~/.config/stencila/agents/` if the agent is not found in the workspace.
- **Hard-coded model or provider**: Flag as a warning, not a failure. Hard-coding reduces portability but may be intentional.
- **Unknown frontmatter fields**: Flag any fields not in the Agent schema as warnings — they may be typos or unsupported properties that will be silently ignored.
- **User asks to fix issues**: If the user asks you to apply suggestions, make the changes, then validate with `stencila agents validate <agent-name>` before reporting completion.

## Validation

When applying suggested improvements, validate the agent before reporting completion:

```sh
# By agent name
stencila agents validate <agent-name>

# By directory path
stencila agents validate .stencila/agents/<agent-name>

# By AGENT.md path
stencila agents validate .stencila/agents/<agent-name>/AGENT.md
```

Validation should pass before you report the changes as complete.

## Limitations

- This skill reviews the *structure, quality, and configuration* of an agent definition. It does not test the agent's runtime behavior or execute it against real inputs.
- The review checks tool names against known Stencila tools but cannot verify that third-party MCP server IDs are valid.
- Security assessment is based on configuration analysis, not runtime behavior.
