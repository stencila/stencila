---
title: Configuration Reference
description: Full reference for SKILL.md frontmatter properties.
---

This page documents all properties available in the YAML frontmatter of a `SKILL.md` file, as well as the Markdown body.

## Required Properties

### `name`

**Type:** `string` — **Required**

The name of the skill. Must be lowercase kebab-case: 1–64 characters, only lowercase alphanumeric characters and hyphens, no leading/trailing/consecutive hyphens. Must match the parent directory name.

Pattern: `^[a-z0-9]([a-z0-9-]{0,62}[a-z0-9])?$`

By convention, names follow a `thing-activity` pattern (e.g. `code-review`, `data-analysis`, `site-design`).

```yaml
name: data-analysis
```

### `description`

**Type:** `string` — **Required**

A description of what the skill does and when to use it. Must be non-empty and at most 1,024 characters. This is the primary text agents use to decide whether to activate a skill, so include specific keywords that help identify relevant tasks.

Good example:

```yaml
description: Analyze datasets and generate summary statistics. Use when working with CSV, Parquet, or database query results.
```

Poor example:

```yaml
description: Helps with data.
```

## Optional Properties

### `license`

**Type:** `string`

The license applied to the skill. Can be an SPDX identifier or a reference to a bundled license file.

```yaml
license: Apache-2.0
```

```yaml
license: Proprietary. LICENSE.txt has complete terms
```

### `compatibility`

**Type:** `string`

Environment requirements for the skill. Must be 1–500 characters if provided. Indicates intended product, required system packages, network access needs, etc.

```yaml
compatibility: Requires Python 3.10+ with pandas and matplotlib
```

```yaml
compatibility: Designed for Claude Code (or similar products). Requires git and docker.
```

Most skills do not need this field.

### `allowed-tools`

**Type:** `string` (space-delimited list)

Pre-approved tools the skill may use. This field is **experimental** per the Agent Skills Specification — support may vary between implementations.

```yaml
allowed-tools: Bash(python:*) Bash(git:*) Read
```

### `metadata`

**Type:** `object` (string keys, string values)

Arbitrary key-value metadata. Use this for additional properties not defined by the spec.

```yaml
metadata:
  author: my-org
  version: "1.0"
```

## Markdown Body

The content after the frontmatter closing `---` is the skill's **instructions**. There are no format restrictions — write whatever helps agents perform the task effectively.

```markdown
---
name: code-review
description: Review code for correctness, style, and security issues.
---

When reviewing code, follow these steps:

1. Read the changed files and understand the context
2. Check for correctness — logic errors, off-by-one, null handling
3. Check for security — injection, authentication, data exposure
4. Check for style — naming, structure, consistency with the codebase
5. Suggest improvements with concrete code examples

## Output Format

Organize findings by severity:

- **Critical** — bugs or security issues that must be fixed
- **Warning** — potential problems or code smells
- **Suggestion** — style improvements or alternative approaches
```

### Writing guidelines

- **Keep it under 500 lines** — the entire body is loaded at once when activated
- **Use step-by-step instructions** — numbered lists are easy for models to follow
- **Include examples** — show expected inputs and outputs
- **Cover edge cases** — note common pitfalls and how to handle them
- **Split long content** — move detailed reference material to `references/` files and reference them with relative paths

## Stencila Extensions

Stencila extends the Agent Skills Specification for better interoperability:

### `allowed-tools` as array

The spec defines `allowed-tools` as a space-delimited string. Stencila parses it into an array of strings internally for easier programmatic use. In YAML frontmatter, write it as a space-delimited string per the spec:

```yaml
allowed-tools: Bash(python:*) Read
```

### `license` mapping

The spec's singular `license` field maps to the `licenses` array inherited from `CreativeWork` in the Stencila schema. The `license` alias is handled automatically — write `license` in your frontmatter and it populates the correct field.

### `metadata` translation

The spec nests properties like `author` and `version` under a `metadata:` object. Stencila hoists these to top-level fields on decode (so they populate `CreativeWork` properties like `authors` and `version`), and nests them back under `metadata:` on encode (so exported skills conform to the spec).

## Deviations from Spec

These are intentional differences from the upstream Agent Skills Specification:

### Workspace-scoped

Skills live in `.stencila/skills/` (or provider-specific directories) within a workspace, not a global directory. This keeps skills project-local and version-controllable with the repository.

### ASCII-only names

The spec says "unicode lowercase alphanumeric characters" but the parenthetical character class (`a-z` and `-`) and all examples use ASCII only. Stencila enforces ASCII and measures length in bytes (equivalent to character count for ASCII). This may be relaxed if the upstream spec clarifies Unicode intent.
