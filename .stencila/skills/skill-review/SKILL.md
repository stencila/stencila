---
name: skill-review
description: Critically review a workspace skill and suggest improvements. Use when asked to review, audit, critique, evaluate, or improve a SKILL.md file or skill directory. Covers frontmatter validation, instruction clarity, completeness, and adherence to the Agent Skills Specification.
keywords:
  - skill
  - review
  - audit
  - critique
  - evaluate
  - improve
  - SKILL.md
allowed-tools: read_file glob grep shell
---

## Overview

Review an existing workspace skill for quality, correctness, completeness, and self-containment. Produce a structured report with specific, actionable suggestions for improvement. The review covers frontmatter fields, instruction clarity, step structure, examples, edge cases, adherence to the Agent Skills Specification, and whether the skill avoids depending on files outside its own directory.

## Steps

1. Identify the skill to review from the user's request — accept a skill name, a directory path, or a `SKILL.md` file path
2. Resolve the skill file: if given a name, look for `.stencila/skills/<name>/SKILL.md` walking up from the current directory; if given a path, use it directly
3. Read the full `SKILL.md` file and any supporting files in the skill directory (`scripts/`, `references/`, `assets/`)
4. Check whether the skill refers to documentation, specifications, or other content outside the skill directory; if it does, assess whether that material should be copied, summarized, or excerpted into local `references/` files instead
5. Evaluate the skill against each criterion in the Review Checklist below
6. Produce a structured review report with a summary, per-criterion findings, and a prioritized list of suggestions
7. If the user asks you to apply the improvements, make the changes and validate the result with `stencila skills validate <skill-name>`

## Review Checklist

### Frontmatter

- **name**: present, matches directory name, valid kebab-case (`^[a-z0-9]([a-z0-9-]{0,62}[a-z0-9])?$`)
- **description**: present, under 1,024 characters, specific (not vague), includes keywords that help agents match the skill to user requests
- **Optional fields**: `license`, `compatibility`, `allowed-tools`, `metadata` — check for correctness if present (e.g., valid SPDX identifier, `compatibility` under 500 characters, `allowed-tools` is space-delimited)

### Discovery and Delegation Metadata

- **keywords**: if present, check that keywords are relevant, not redundant with the description, and include likely user intent words, artifact types, and domain terms. Flag generic or overly broad keywords. If absent, recommend adding keywords to improve discoverability
- **Coherence check**: verify that `description` and `keywords` work together — they should be complementary, not redundant. Flag cases where the same text appears verbatim in multiple fields

### Structure and Clarity

- Uses step-by-step numbered lists that are easy for a model to follow
- Steps are in a logical order with no missing or circular dependencies
- Language is imperative and unambiguous (e.g., "Read the file" not "You might want to read the file")
- No orphaned sections or dangling references to files that don't exist

### Completeness

- Includes concrete input/output examples
- Covers edge cases and common pitfalls
- No placeholder content (`TODO`, `<placeholder>`, or empty sections)
- References to external files (`scripts/`, `references/`, `assets/`) point to files that actually exist in the skill directory

### Self-Containment and References

- The skill is self-contained and does not depend on reading files outside its own directory
- `SKILL.md` does not send agents to repo documentation, specs, or other external files outside the skill directory for essential instructions
- If outside material is needed, the skill includes a local summary, excerpt, or copy in `references/`
- Reference files are focused and appropriately scoped rather than large catch-all documents
- Links and relative paths point only to files within the skill directory

### Size and Focus

- Body is under 500 lines / 5,000 tokens
- Skill has a single clear purpose — not trying to do too many things
- Detailed reference material is moved to `references/` files rather than inlined
- Individual reference files stay focused so agents can load only the context they need

### Consistency

- Formatting is consistent (heading levels, list styles, code block languages)
- Terminology is used consistently throughout
- Conventions match other skills in the same workspace

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

Input: "Review the skill-creation skill"

Process:

1. Resolve to `.stencila/skills/skill-creation/SKILL.md`
2. Read the file and check for supporting files in the directory (`scripts/`, `references/`, `assets/`)
3. Check whether it relies on documentation outside the skill directory or whether any needed material has been localized into `references/`
4. Evaluate frontmatter: `name` is `skill-creation`, matches directory, valid kebab-case; `description` is specific and under 1,024 characters
5. Evaluate structure, completeness, self-containment, size, and consistency against the checklist
6. Run `stencila skills validate skill-creation` to confirm the skill is valid
7. Produce the report below

Output (use `###` headings in the report):

> ### Summary
>
> The skill-creation skill is well-structured with comprehensive coverage of naming rules, file format, and validation. Two minor improvements are possible.
>
> ### Findings
>
> | Area | Status | Notes |
> |------|--------|-------|
> | Frontmatter | ✅ Pass | Name and description are valid and specific |
> | Structure | ✅ Pass | Clear numbered steps in logical order |
> | Completeness | ⚠️ Warning | Template section serves as an example but a concrete before/after example would be clearer |
> | Size and Focus | ✅ Pass | Well within size limits, single purpose |
> | Consistency | ✅ Pass | Consistent formatting throughout |
>
> ### Suggestions
>
> 1. Add a concrete example showing a complete user request and the resulting `SKILL.md` file, beyond the generic template
> 2. Consider adding an `allowed-tools` field to pre-approve `write_file`, `shell`, and `read_file`

## Edge Cases

- **Skill not found**: If the skill cannot be located, report the error clearly and suggest checking the name or path. List available skills if possible using `stencila skills list` or by listing `.stencila/skills/` directories.
- **Multiple skills requested**: Review each skill separately with its own report section. Ask the user to confirm if reviewing all skills is intended.
- **Skill has no body content**: Flag this as a failure — a skill with only frontmatter is incomplete.
- **Supporting files are large**: For files in `scripts/`, `references/`, or `assets/`, check that they exist and are referenced from `SKILL.md`, but do not reproduce their full content in the report.
- **Skill refers outside itself**: Flag references to files or docs outside the skill directory as a self-containment issue. Recommend moving the necessary material into focused files under `references/` and updating links to those local files.
- **User asks to fix issues**: If the user asks you to apply suggestions, make the changes, then validate with `stencila skills validate <skill-name>` before reporting completion.

## Validation

When applying suggested improvements, validate the skill before reporting completion:

```sh
# By skill name
stencila skills validate <skill-name>

# By directory path
stencila skills validate .stencila/skills/<skill-name>

# By SKILL.md path
stencila skills validate .stencila/skills/<skill-name>/SKILL.md
```

Validation should pass before you report the changes as complete.

## Limitations

- This skill reviews the *structure and quality* of a skill definition. It does not review the correctness or security of code in `scripts/` files — only that they exist and are referenced from `SKILL.md`.
- The review does not execute the skill or test it against real inputs.
