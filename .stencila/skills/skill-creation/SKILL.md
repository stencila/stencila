---
name: skill-creation
description: Create a new Stencila workspace skill. Use when asked to create, write, or scaffold a SKILL.md file or skill directory.
allowed-tools: read_file write_file glob grep shell
---

## Overview

Create a new workspace skill directory and `SKILL.md` file following the [Agent Skills Specification](https://agentskills.io/specification). A skill is a directory under `.stencila/skills/` containing a `SKILL.md` file with YAML frontmatter and a Markdown body. Skills are reusable instruction sets for AI agents.

## Steps

1. Determine the skill name from the user's request
2. Validate the name against the naming rules below
3. Resolve the closest workspace skill directory: walk up from the current directory to find the nearest directory containing `.stencila/`, or use the repository root if none exists
4. Create the directory `<closest-workspace>/.stencila/skills/<name>/`
5. Write the `SKILL.md` file with frontmatter and instructions — include activation keywords in the `description` so agents can match the skill to user requests
6. Replace placeholders such as `TODO` before considering the skill complete
7. Optionally create `scripts/`, `references/`, or `assets/` subdirectories if the skill needs them
8. Validate the finished skill with `stencila skills validate <name>`, the skill directory path, or the `SKILL.md` path

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
- `allowed-tools` — space-delimited list of pre-approved tools (e.g., `read_file grep shell`)
- `metadata` — arbitrary key-value pairs (e.g., `author`, `version`)

## Template

Use this as a starting point:

```markdown
---
name: <skill-name>
description: <Clear description including keywords that help agents match this skill to user requests. Do not leave placeholders such as TODO. Max 1,024 characters.>
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

## Writing Guidelines

- Keep the body under 500 lines / 5,000 tokens
- Use step-by-step numbered lists — easy for models to follow
- Include input/output examples
- Cover edge cases and common pitfalls
- Move detailed reference material to `references/` files
- Do not leave placeholder frontmatter or body content such as `TODO`
- Write a description that is specific, not vague (e.g., "Analyze datasets and generate summary statistics. Use when working with CSV, Parquet, or database query results." not "Helps with data.")
- Keep `description` under 1,024 characters and `compatibility` under 500 characters

## Edge Cases

- **Skill directory already exists**: Ask the user whether to overwrite, merge, or abort before modifying an existing skill. Never silently overwrite.
- **Name mismatch**: If the user provides a name that doesn't match kebab-case rules, suggest a corrected version rather than failing silently.
- **Nested workspaces**: If multiple `.stencila/` directories exist in the ancestor chain, use the nearest one. Do not create a duplicate `.stencila/skills/` tree.
- **Empty or placeholder content**: Do not consider the skill complete if any `TODO`, `<placeholder>`, or empty sections remain in the final `SKILL.md`.

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
