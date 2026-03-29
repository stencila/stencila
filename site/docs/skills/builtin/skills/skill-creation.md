---
title: "Skill Creation Skill"
description: "Create a new Stencila workspace skill. Use when asked to create, write, or scaffold a SKILL.md file or skill directory."
keywords:
  - skill
  - create
  - scaffold
  - create
  - write
  - SKILL.md
---

Create a new Stencila workspace skill. Use when asked to create, write, or scaffold a SKILL.md file or skill directory.

**Keywords:** skill · create · scaffold · create · write · SKILL.md

# Configuration

| Property | Value |
| -------- | ----- |
| Allowed tools | `read_file`, `write_file`, `edit_file`, `apply_patch`, `glob`, `grep`, `shell`, `ask_user` |

# Instructions

## Overview

Create a new workspace skill directory and `SKILL.md` file following the Agent Skills Specification. A skill is a directory under `.stencila/skills/` containing a `SKILL.md` file with YAML frontmatter and a Markdown body. Skills are reusable instruction sets for AI agents.

Skills should be self-contained. Do not rely on documentation or other content outside the skill directory. If the skill needs supporting material from elsewhere in the repository or from another source, copy it, summarize it, or excerpt it into files inside the skill's own `references/` directory, then link to those local files from `SKILL.md`.

## Steps

1. Determine the skill name from the user's request
2. Validate the name against the naming rules below
3. Resolve the closest workspace skill directory: walk up from the current directory to find the nearest directory containing `.stencila/`, or use the repository root if none exists
4. Create the directory `<closest-workspace>/.stencila/skills/<name>/`
5. Write the `SKILL.md` file with frontmatter and instructions — include activation keywords in the `description` so agents can match the skill to user requests
6. Add `keywords` to the frontmatter to improve discoverability and delegation accuracy — include terms reflecting likely user intents, artifacts, and domains
7. Replace placeholders such as `TODO` before considering the skill complete
8. If the skill depends on supporting guidance, examples, or specifications, create focused files in `references/` and put that material there rather than referring to files elsewhere in the repo
9. Optionally create `scripts/`, `references/`, or `assets/` subdirectories if the skill needs them
10. Validate the finished skill with `stencila skills validate <name>`, the skill directory path, or the `SKILL.md` path

When working from a nested directory in a repository, create the skill in the closest workspace's `.stencila/skills/` directory rather than creating a new `.stencila/` tree under the current subdirectory.

## Naming Rules

Skill names must be **lowercase kebab-case**:

- 1–64 characters
- Only lowercase alphanumeric characters and hyphens (`a-z`, `0-9`, `-`)
- Must not start or end with a hyphen
- Must not contain consecutive hyphens (`--`)
- Must match the parent directory name
- Pattern: `^[a-z0-9]([a-z0-9-]{0,62}[a-z0-9])?$`

By convention, names follow a `thing-activity` pattern describing the domain and action (e.g., `code-review`, `data-analysis`, `test-generation`).

Common corrections: `reviewCode` → `code-review`, `data_analysis` → `data-analysis`, `Test-Gen` → `test-gen`.

## SKILL.md Format

The file has two parts:

1. **YAML frontmatter** between `---` delimiters
2. **Markdown body** with instructions for the agent

### Required frontmatter fields

- `name` — the skill name (must match directory name)
- `description` — what the skill does and when to use it (max 1,024 characters). Include specific keywords that help agents decide whether to activate the skill.

### Optional frontmatter fields

- `license` — SPDX identifier or reference to a license file
- `compatibility` — environment requirements (max 500 characters)
- `allowed-tools` — space-delimited or comma-delimited list of pre-approved tools (e.g., `read_file grep shell` or `read_file, grep, shell, ask_user`).
- `keywords` — (Stencila extension) list of keywords or tags for discovery and routing. Use terms that reflect likely user intents, artifacts, and domains. Helps managers and selection systems find and rank this skill. Include both positive signals (what this skill does) and negative signals (what it doesn't do) as keywords.
- `metadata` — arbitrary key-value pairs (e.g., `author`, `version`)

## Template

Use this as a starting point:

```markdown
---
name: <skill-name>
description: <Clear description including keywords that help agents match this skill to user requests. Do not leave placeholders such as TODO. Max 1,024 characters.>
keywords:
  - <keyword1>
  - <keyword2>
# license: MIT
# allowed-tools: read_file grep shell
# metadata:
#   author: <name>
#   version: 0.1.0
---

## Steps

1. <First step>
2. <Second step>
3. <Third step>

## Examples

Input: <describe expected input>

Output: <describe expected output>

## Edge Cases

- <Common pitfall and how to handle it>
```

## Directory Structure

Each skill gets its own subdirectory. Only `SKILL.md` is required:

```
.stencila/skills/
  <skill-name>/
    SKILL.md              # Required — frontmatter + instructions
    scripts/              # Optional — executable code
    references/           # Optional — additional documentation
    assets/               # Optional — static resources
```

Use `scripts/` for executable code, `references/` for detailed docs loaded on demand, and `assets/` for templates and data files. Reference them from `SKILL.md` using relative paths.

Do not point `SKILL.md` at repository documentation, specifications, or other files outside the skill directory. When outside material is necessary, prefer adding a concise summary or excerpt under `references/` instead of copying a large document verbatim. Keep individual reference files focused so agents can load only the minimum context needed.

## Choosing `allowed-tools`

Only include tools the skill genuinely needs; prefer the minimal set.

| Tool                       | Use for                                          | Include when                                                                |
| -------------------------- | ------------------------------------------------ | --------------------------------------------------------------------------- |
| `read_file`                | Read existing files                              | The skill needs to inspect repository or workspace content                  |
| `write_file`               | Create new files or overwrite whole files        | The skill creates files from scratch                                        |
| `apply_patch`, `edit_file` | Modify existing files in place                   | The skill updates existing files; some models prefer one or the other       |
| `grep`                     | Search file contents                             | The skill needs to find patterns, symbols, or references                    |
| `glob`                     | Find files by pattern                            | The skill needs to discover files or directories                            |
| `web_fetch`                | Fetch and save web content locally               | The skill needs to retrieve web pages or external documentation for review or summarization |
| `shell`                    | Run commands                                     | The skill needs validation, formatting, tests, or other command-line checks |
| `ask_user`                 | Request clarification, confirmation, or approval | The skill may need user feedback before proceeding                          |

## Writing Guidelines

- Keep the body under 500 lines / 5,000 tokens
- Use step-by-step numbered lists — easy for models to follow
- Include input/output examples
- Cover edge cases and common pitfalls
- Move detailed reference material to `references/` files
- Keep the skill self-contained: avoid links or instructions that depend on files outside the skill directory
- If external or repo-local guidance is needed, summarize or excerpt it into focused files under `references/`
- Do not leave placeholder frontmatter or body content such as `TODO`
- Write a description that is specific, not vague (e.g., "Analyze datasets and generate summary statistics. Use when working with CSV, Parquet, or database query results." not "Helps with data.")
- Keep `description` under 1,024 characters and `compatibility` under 500 characters

### Keeping skills workflow-agnostic

Skills must describe **generic domain competence with generic inputs and outputs**. They should work equally well when invoked by a user in a chat, by an agent acting alone, or by a workflow stage prompt. Workflow-specific concerns — context keys, route labels, `workflow_*` tool calls — belong in the workflow's stage prompts, not in skills.

Follow these rules:

- **Do not reference `workflow_get_context`, `workflow_set_context`, `workflow_set_route`, or `workflow_get_output`** in any skill. These are workflow orchestration tools. If a skill needs input data, declare it in a "Required Inputs" table and let the caller (user, agent prompt, or workflow stage prompt) supply it.
- **Do not define "Context Keys" or "Route Labels" tables** in skills. The workflow owns its data contract — which keys hold which values, and which labels control which branches. Skills should not know or care about that contract.
- **Do not reference workflow node names** (e.g., `RunTestsRed`, `CheckRemaining`) or specific workflow files. Skills should not know which workflow is calling them.
- **Declare inputs and outputs generically.** Use a "Required Inputs" table listing what the skill needs (with Required/Optional) and an "Outputs" table listing what it produces. Use domain-appropriate names (e.g., "Acceptance criteria", "Test files", "Recommendation") rather than context key names.
- **Use a single sentence to explain how inputs arrive.** After the inputs table, include: "When used standalone, these inputs come from the user or the agent's prompt. When used within a workflow, the workflow's stage prompt will specify how to obtain them." This one sentence bridges both use cases without coupling to either.
- **Mark plan/file fallbacks as standalone convenience.** If a skill falls back to reading from a well-known location (e.g., `.stencila/plans/`) when an input is missing, frame this as a convenience for standalone use: "attempt to infer from X as a standalone convenience. In workflow use, the stage prompt should provide this explicitly."

The workflow's stage prompts are the glue layer. A well-structured stage prompt follows this pattern:

1. **Read workflow state** — `workflow_get_context` / `workflow_get_output` calls
2. **Delegate to the skill** — pass the retrieved values as the skill's declared inputs
3. **Store the skill's outputs** — `workflow_set_context` with the workflow's chosen key names
4. **Route** — `workflow_set_route` with the workflow's chosen labels, mapped from the skill's domain outputs

This separation means skills stay reusable across workflows, agents can use skills without workflow infrastructure, and the workflow is the single source of truth for its own data contract.

## Edge Cases

- **Skill directory already exists**: Ask the user whether to overwrite, merge, or abort before modifying an existing skill. Never silently overwrite.
- **Name mismatch**: If the user provides a name that doesn't match kebab-case rules, suggest a corrected version rather than failing silently.
- **Nested workspaces**: If multiple `.stencila/` directories exist in the ancestor chain, use the nearest one. Do not create a duplicate `.stencila/skills/` tree.
- **Empty or placeholder content**: Do not consider the skill complete if any `TODO`, `<placeholder>`, or empty sections remain in the final `SKILL.md`.
- **External dependencies in documentation**: If instructions refer to docs or files outside the skill directory, move the required content into focused files under `references/` and update `SKILL.md` to point only to those local copies, summaries, or excerpts.

## Validation

Before finishing, validate the skill:

```sh
# By skill name
stencila skills validate <skill-name>

# By directory path
stencila skills validate .stencila/skills/<skill-name>

# By SKILL.md path
stencila skills validate .stencila/skills/<skill-name>/SKILL.md
```

Validation should pass before you report the skill as complete.

---

This page was generated from [`.stencila/skills/skill-creation/SKILL.md`](https://github.com/stencila/stencila/blob/main/.stencila/skills/skill-creation/SKILL.md).
