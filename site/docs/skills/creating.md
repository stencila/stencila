---
title: Creating Skills
description: How to create workspace skills with SKILL.md files.
---

## Quick Start

Create a new skill with the CLI:

```sh
stencila skills create data-analysis
```

This creates `.stencila/skills/data-analysis/SKILL.md` in the closest workspace with a template you can edit:

```markdown
---
name: data-analysis
description: TODO
---

TODO: Add instructions for this skill.
```

## The SKILL.md File

A skill is a directory containing a `SKILL.md` file. The file has two parts:

1. **YAML frontmatter** — metadata (name, description, optional fields)
2. **Markdown body** — instructions for the agent

Here is a minimal example:

```markdown
---
name: code-review
description: Review code for correctness, style, and security issues.
---

When asked to review code:

1. Read the files and understand the change
2. Check for correctness and security issues
3. Suggest concrete improvements with code examples
```

And a fully configured example:

```markdown
---
name: data-analysis
description: Analyze datasets and generate summary statistics. Use when working with CSV, Parquet, or database query results.
license: Apache-2.0
compatibility: Requires Python 3.10+ with pandas and matplotlib
allowed-tools: Bash(python:*) Read
metadata:
  author: my-org
  version: "1.0"
---

## Steps

1. Load the dataset using pandas
2. Check for missing values and data types
3. Generate summary statistics
4. Create visualizations for key distributions

## Examples

Input: a CSV file with columns `date`, `value`, `category`

Output: summary table, time series plot, category breakdown

## Edge Cases

- Handle missing values by noting them in the summary, not by dropping rows
- For datasets over 1M rows, sample before plotting
```

## Skill Names

Skill names must be **lowercase kebab-case**:

- 1–64 characters
- Only lowercase alphanumeric characters and hyphens (`a-z`, `0-9`, `-`)
- Must not start or end with a hyphen
- Must not contain consecutive hyphens (`--`)
- Must match the parent directory name

By convention, names follow a `thing-activity` pattern describing the skill's domain and action:

| Name | Domain | Activity |
| ---- | ------ | -------- |
| `code-review` | code | review |
| `data-analysis` | data | analysis |
| `site-design` | site | design |
| `test-generation` | test | generation |
| `doc-writing` | doc | writing |

## Directory Structure

Each skill gets its own subdirectory under a `skills/` directory. The `SKILL.md` file is required; additional directories are optional:

```
.stencila/skills/
  data-analysis/
    SKILL.md              # Required — frontmatter + instructions
    scripts/              # Optional — executable code
      extract.py
      summarize.py
    references/           # Optional — additional documentation
      REFERENCE.md
      pandas-tips.md
    assets/               # Optional — static resources
      template.csv
      report-template.md
```

| Directory | Purpose |
| --------- | ------- |
| `scripts/` | Executable code the agent can run (Python, Bash, JavaScript, etc.) |
| `references/` | Additional documentation loaded on demand |
| `assets/` | Static resources like templates, schemas, and data files |

Reference these files from your `SKILL.md` using relative paths:

```markdown
See [the reference guide](references/REFERENCE.md) for detailed API docs.

Run the extraction script:
scripts/extract.py
```

## Writing Effective Instructions

The Markdown body is loaded in full when an agent activates your skill. Write instructions that help agents complete the task effectively:

- **Keep it focused** — under 500 lines / 5,000 tokens recommended
- **Use step-by-step instructions** — numbered lists are easy for models to follow
- **Include examples** — show expected inputs and outputs
- **Cover edge cases** — note common pitfalls and how to handle them
- **Split long content** — move detailed reference material to `references/` files

The entire `SKILL.md` body is loaded at once. If your skill needs extensive documentation, keep the main file concise and use relative file references for details that the agent can load on demand.

## Provider-Specific Skills

You can place skills in provider-specific directories to tailor instructions for a particular model family:

```
.stencila/skills/
  code-review/
    SKILL.md              # Base skill — used by all providers

.claude/skills/
  code-review/
    SKILL.md              # Override — used by Anthropic agents

.codex/skills/
  code-review/
    SKILL.md              # Override — used by OpenAI agents
```

When the same skill name exists in both `.stencila/skills/` and a provider directory, the provider-specific version takes precedence. This lets you write provider-optimized instructions while maintaining a universal fallback.

## Validation

Validate a skill to check it conforms to the Agent Skills Specification:

```sh
# By skill name
stencila skills validate data-analysis

# By directory path
stencila skills validate .stencila/skills/data-analysis

# By SKILL.md path
stencila skills validate .stencila/skills/data-analysis/SKILL.md
```

Validation checks:

- Name format (kebab-case, 1–64 characters, ASCII only)
- Name matches the parent directory name
- Description is non-empty and not a placeholder (e.g., "TODO")
- Description length (max 1,024 characters)
- Compatibility length (max 500 characters, if provided)
