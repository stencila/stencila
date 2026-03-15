---
name: workflow-review
description: Critically review a Stencila workflow and suggest improvements. Use when asked to review, audit, critique, evaluate, or improve a workflow directory or WORKFLOW.md file. Covers frontmatter validation, DOT pipeline quality, workflow structure, agent selection quality, discovery metadata, ephemeral workflow conventions, workflow composition, and adherence to Stencila workflow patterns.
keywords:
  - workflow
  - review
  - audit
  - critique
  - evaluate
  - improve
  - WORKFLOW.md
allowed-tools: read_file glob grep shell list_agents list_workflows
---

## Overview

Review an existing Stencila workflow for quality, correctness, and completeness. Produce a structured report with specific, actionable suggestions. The review covers frontmatter fields, DOT pipeline structure, workflow design quality, branching and approval logic, agent references and agent-fit quality, discovery metadata, ephemeral workflow conventions, and adherence to the workflow patterns used in this workspace.

## Steps

1. Identify the workflow to review from the user's request — accept a workflow name, a directory path, or a `WORKFLOW.md` file path
2. Resolve the workflow file: if given a name, look for `.stencila/workflows/<name>/WORKFLOW.md` walking up from the current directory; if given a path, use it directly
3. Read the full `WORKFLOW.md` file and inspect the workflow directory for supporting files such as `.gitignore`, `scripts/`, `references/`, and `assets/`
4. When supporting files are present, check that files referenced from `WORKFLOW.md` exist and that existing supporting files are meaningfully referenced where appropriate; for large files, verify existence and relevance without reproducing their full contents in the review
5. When evaluating workspace conventions, compare the workflow against one or two existing workflows in `.stencila/workflows/` or related workflow skills when available
6. When the workflow uses specialized `agent` attributes or agent fit is unclear, use `list_agents` to compare referenced agents against available agents and their metadata rather than guessing from names alone
7. When the workflow composes child workflows with `workflow="name"` or the choice of nested workflows seems questionable, use `list_workflows` to compare referenced child workflows against available workflows and their metadata rather than guessing from names alone
8. Evaluate the workflow against each criterion in the Review Checklist below
9. If the workflow is explicitly intended to be ephemeral, check whether the directory contains a `.gitignore` sentinel file with exactly `*`; if the sentinel is missing or has different contents, treat that as a failure. If the workflow only seems temporary from context, treat a missing sentinel as a warning unless the workflow clearly claims ephemeral status
10. Produce a structured review report with a summary, per-criterion findings, and a prioritized list of suggestions
11. If the user asks you to apply improvements, make the changes and validate the result using the most specific path available first, such as `stencila workflows validate <workflow-dir>` or `stencila workflows validate <workflow-file>`; use name-based validation only when the name is clearly resolved

## Review Checklist

### Frontmatter

- **name**: present, matches directory name, valid kebab-case (`^[a-z0-9]([a-z0-9-]{0,62}[a-z0-9])?$`)
- **name convention**: the name should describe the workflow's end-to-end purpose using `thing-process` or `thing-process-approach`; flag names that overfit the exact graph shape or enumerate too many steps
- **description**: present, concise, specific, and not placeholder text (`TODO`, `<placeholder>`)
- **goal-hint**: hint text displayed across user interfaces (TUI, web, email, Slack, etc.) when the workflow is activated, guiding the user to provide a specific goal. Most workflows should include this because most workflows expect the user to supply their own objective. If present, it should be a concise, actionable question (e.g., "What kind of data analysis do you want to perform?"). Flag hint text that is generic or unhelpful. If absent on a workflow that clearly expects user-supplied goals, recommend adding it
- **goal**: optional, but if present it should express a stable, fixed objective and be meaningfully distinct from `description`. Flag generic goals that merely restate the workflow's purpose (e.g., "Produce an acceptable X for the requested purpose") — they provide no runtime value and clutter the user interface. Recommend replacing them with `goal-hint` or removing them entirely
- **Optional fields**: `license`, `compatibility`, `metadata` — check for correctness if present (e.g., `compatibility` under 500 characters)
- **Unknown fields**: flag frontmatter fields that do not match the workflow conventions in this workspace as warnings, especially custom fields that appear to duplicate executable configuration already represented in the DOT graph

When reviewing names, apply these conventions:

- prefer `thing-process` for the default case, for example `code-review` or `blog-generation`
- prefer `thing-process-approach` when different workflows for the same process need different tradeoff signals, for example `code-generation-iterative` or `agent-creation-guided`
- treat `approach` as a broad strategy or cost/quality tradeoff, not as a literal summary of the graph
- warn on names that try to encode the entire pipeline shape, such as `create-review-refine-test-deploy`, unless there is a strong workspace-specific reason for that style

### DOT Pipeline Structure

- The Markdown body should begin with a short human-readable explanation of the workflow before the first `dot` fenced code block; flag workflows that jump straight into the DOT block without any introductory prose
- The first `dot` fenced code block in the body contains a valid-looking directed graph such as `digraph name { ... }`
- The workflow has a clear executable path from start to finish
- Node and edge names are readable and internally consistent
- Prompts, `agent` attributes, conditions, and labels are attached to the correct nodes or edges
- The DOT source follows the workspace house style: keep the `Start -> ...` entry edge near the top, then colocate each node definition with its outgoing edge or edges where practical
- The graph is not missing obvious terminal or branching connections
- Only the first `dot` block is relied on for execution; additional DOT blocks, if any, are treated as documentation and should not create ambiguity
- When a node uses `workflow="name"`, treat it as a composed child workflow node and check that the composition boundary is clear and justified; this attribute is normalized to workflow-handler semantics rather than acting like a normal LLM, shell, or branch node

### Workflow Design Quality

- The sequence of steps makes progress toward the stated goal rather than repeating generic prompting
- The workflow breaks non-trivial tasks into meaningful stages with clear outputs or decision points
- Branches, loops, and approval gates are used only where they add value
- Revision loops have a plausible feedback path rather than a vague cycle
- Human review (`shape=human`) is used appropriately for approval, oversight, or trust-boundary decisions
- Multi-question interviews via `interview-ref` are used appropriately — combining a routing decision with structured feedback in a single review pause, not overloading a single interview with unrelated questions that would be clearer as separate human nodes
- When an interview uses `show-if`, check that the referenced `store` key belongs to an earlier question and the condition syntax is valid (`"store_key == value"` or `"store_key != value"`); flag `show-if` conditions that reference a `store` key from the same or a later question
- When an interview uses `finish-if`, check that it is on a `yes-no`, `confirm`, or `single-select` question — `finish-if` is not supported on `freeform` or `multi-select` questions; verify that the early exit value makes sense for the interview flow and that questions after the gate are ones the user would genuinely want to skip
- Workflow composition is used appropriately — child workflows should encapsulate meaningful reusable subprocesses, not obscure simple local steps without a readability or reuse benefit
- Parent workflows should focus on orchestration, while child workflows should own detailed internal execution where that split improves clarity
- Top-down design is a valid approach: a workflow may intentionally reference agents or child workflows that do not yet exist, planning to create them later. When a workflow appears to follow this pattern, evaluate the pipeline structure and stage responsibilities on their own merits rather than penalizing unresolved references; instead note which dependencies remain to be created
- The workflow is no more complex than necessary for the task

### Agents, Workflows, and Prompts

#### Agent references

- Referenced agent names should be plausible; use `list_agents` to confirm availability and metadata fit when uncertain. Forward-referencing agents that do not yet exist is valid in top-down design — the runtime falls back to a default agent, so flag missing agents as informational dependencies rather than errors
- Agent selection should be justified by metadata, not name alone: `description` should match the task, `keywords` should overlap domain terms, `when-to-use` should provide positive signals, and `when-not-to-use` should not conflict with the node's role
- The workflow does not overuse `agent.*` overrides where simpler inline attributes would be clearer
- A workflow with no `agent` attributes may be valid, but note when explicit agent selection would materially improve clarity or control

#### Child workflow references

- For `workflow="name"` nodes: check that the child workflow name is plausible (use `list_workflows` to compare alternatives when composition matters); check that the child goal is passed clearly via `goal="..."` or a sensible default from `prompt`/`label`; and flag missing child workflows as outstanding dependencies in top-down designs rather than errors
- Child workflow selection should also be justified by metadata, not name alone
- Recommend nesting a child workflow only when it encapsulates a meaningful reusable subprocess, improves readability, or cleanly separates orchestration from execution; flag trivial or over-engineered nesting

#### Prompts and content refs

- Prompts are specific enough to guide each node's local task
- When the workflow uses `prompt-ref`, `shell-ref`, `ask-ref`, or `interview-ref`, referenced ids exist in code blocks or code chunks in the same `WORKFLOW.md`, are unique, and are used where they improve readability, typically for long or multiline content rather than short single-line values
- If `goal` is present, prompts should use `$goal` consistently. If `goal` is absent but prompts reference `$goal`, check that `goal-hint` is set (the user's response becomes `$goal` at runtime)
- The workflow does not overuse content refs where simpler inline attributes would be clearer

#### Interview specs

- When a workflow uses `interview-ref`, check that the referenced YAML block is valid interview spec YAML (has a `questions` array with at least one entry, each question has `question` text and a recognized `type`)
- Check that freeform questions in the interview spec have `store` keys — a freeform question without `store` collects an answer that is never stored, which is almost certainly a mistake
- Check that the routing question (first `single-select`) has option labels matching the outgoing edge labels from the human node
- Check that `show-if` conditions reference valid `store` keys from earlier questions and use the correct syntax; flag conditions that would always be true, always be false, or reference non-existent keys
- Check that `finish-if` is only used on supported question types (`yes-no`, `confirm`, `single-select`) and that the early-exit value is a valid answer for that question type (e.g., `"yes"` or `"no"` for `yes-no`, an option label for `single-select`)
- Flag `interview-ref` used for a single simple question where `ask` or `ask-ref` would be simpler

### Ephemeral Workflow Conventions

- If the workflow is explicitly intended to be ephemeral, the directory contains a `.gitignore` file with exactly `*`
- If the workflow is permanent, it should not use custom frontmatter like `ephemeral: true` or other non-standard markers to indicate temporary status
- Report whether the ephemeral/permanent status is clear from the directory contents and workflow context
- If the workflow only appears temporary by context but does not explicitly claim ephemeral status, a missing sentinel is usually a warning rather than a failure

### Discovery and Delegation Metadata

- **keywords**: if present, check that keywords are relevant, not redundant with the description, and include likely user-intent words and domain terms. Flag generic or overly broad keywords. If absent, recommend adding keywords to improve discoverability
- **when-to-use / when-not-to-use**: if present, check that entries are specific, actionable, and complementary to the description rather than duplicating it. Flag vague signals. If absent, recommend adding them to improve manager delegation accuracy
- **Coherence check**: verify that `description`, `keywords`, `when-to-use`, and `when-not-to-use` work together — they should be complementary, not redundant
- **Agent and composition coherence**: check that agent and child workflow assignments are consistent with their metadata (see Agents and Prompts above). For composition, check that the parent description, node names, and child workflow purposes fit together coherently; use `list_workflows` when needed to compare alternatives

### Completeness and Clarity

- The file has no placeholder content (`TODO`, `<placeholder>`, empty sections)
- The body begins with a short human-readable explanation before the DOT block, and any additional Markdown documentation after the DOT block supports the workflow and does not contradict it
- References to supporting files in `scripts/`, `references/`, or `assets/` point to files that actually exist
- References from DOT to reusable fenced code blocks via `prompt-ref`, `shell-ref`, `ask-ref`, and `interview-ref` point to ids that actually exist in the same file
- References from DOT to composed child workflows via `workflow="name"` point to workflows that exist when the review scope allows that to be checked; if child workflows do not yet exist, note them as outstanding dependencies for top-down designs rather than treating them as errors
- When supporting files are large, check existence and relevance without reproducing their full contents in the report
- The workflow is understandable to both the execution engine and a human reader

### Consistency

- Formatting is consistent (heading levels, list styles, code block languages)
- Terminology is used consistently throughout
- Conventions match other workflows and workflow-related skills in the same workspace; compare against one or two nearby examples when available
- DOT organization is easy to scan: entry edge first, then node-local blocks instead of a large separated edges section and nodes section
- The workflow's frontmatter, Markdown explanation, and DOT graph do not contradict each other
- The workflow name aligns with current workspace naming guidance and, when present, any approach modifier is used consistently with similar workflows
- If the workflow uses composition, parent and child workflows use consistent terminology for the subprocess being delegated

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

### Outstanding Dependencies (when applicable)

When the workflow references agents or child workflows that do not yet exist, include a section listing these as dependencies to be created. This is expected in top-down workflow design and should be presented as an informational inventory, not as errors.

Use heading level 3 (`###`) for each section in your output.

## Examples

Input: "Review the code-review workflow"

Process:

1. Resolve to `.stencila/workflows/code-review/WORKFLOW.md`
2. Read the file and inspect the workflow directory for `.gitignore` and any supporting files
3. Evaluate frontmatter: `name` is `code-review`, matches the directory, valid kebab-case, and follows the recommended `thing-process` convention; `description` and `goal` are specific
4. Check the first `dot` block: it defines a directed graph with a clear path through design, build, test, and review
5. Evaluate design quality: the test step gates progression and the review step provides a meaningful approval decision
6. Run `stencila workflows validate .stencila/workflows/code-review` if applying fixes
7. Produce the report below

Output (use `###` headings in the report):

> ### Summary
>
> The code-review workflow is well-structured and easy to follow, with a sensible test-and-review gate before completion. One small improvement would make its intent even clearer.
>
> ### Findings
>
> | Area | Status | Notes |
> |------|--------|-------|
> | Frontmatter | ✅ Pass | Name, description, and goal are valid and specific; the name follows the `thing-process` convention |
> | DOT pipeline structure | ✅ Pass | First code block is a clear directed graph with readable nodes and branches |
> | Workflow design quality | ✅ Pass | The test and review gates provide meaningful evaluation steps |
> | Agents and prompts | ⚠️ Warning | Some nodes rely on generic prompts and could be more specific |
> | Ephemeral conventions | ✅ Pass | Workflow is permanent and does not use non-standard temporary markers |
> | Completeness and clarity | ✅ Pass | No placeholders; documentation matches the graph |
> | Consistency | ✅ Pass | Formatting and terminology are consistent |
>
> ### Suggestions
>
> 1. Make the `Test` node prompt more specific about what to validate and how to report failure, so the revision loop gets more actionable feedback

Input: "Review the temporary note-summary workflow"

Process:

1. Resolve to `.stencila/workflows/note-summary/WORKFLOW.md`
2. Inspect the directory and detect `.gitignore` containing exactly `*`, indicating an ephemeral workflow
3. Evaluate frontmatter and the first `dot` block
4. Check that the workflow does not try to encode ephemeral status in frontmatter
5. Produce the report

Output (use `###` headings in the report):

> ### Summary
>
> The note-summary workflow is a good minimal ephemeral workflow. Its temporary status is correctly indicated by the `.gitignore` sentinel rather than custom frontmatter.
>
> ### Findings
>
> | Area | Status | Notes |
> |------|--------|-------|
> | Frontmatter | ✅ Pass | Required fields are present and specific |
> | DOT pipeline structure | ✅ Pass | Simple linear graph is appropriate for the task |
> | Workflow design quality | ✅ Pass | Minimal but sufficient for a one-step summary workflow |
> | Agents and prompts | ✅ Pass | Prompt is concise and aligned with the goal |
> | Ephemeral conventions | ✅ Pass | `.gitignore` sentinel contains exactly `*` |
> | Completeness and clarity | ✅ Pass | No placeholders or contradictory documentation |
> | Consistency | ✅ Pass | Matches workspace workflow conventions |
>
> ### Suggestions
>
> 1. Optionally add one sentence of Markdown documentation for human readers describing when to save versus discard this temporary workflow

Input: "Review the deploy-helper workflow"

Process:

1. Resolve to `.stencila/workflows/deploy-helper/WORKFLOW.md`
2. Read the file — frontmatter includes `ephemeral: true`; the first code block is Markdown or plain text instead of `dot`; the graph has a `Review -> Review` self-loop with no feedback node, prompts are vague, and the name is generic rather than process-oriented
3. Inspect the directory and note there is no `.gitignore` sentinel file
4. Evaluate the workflow against the checklist
5. Produce the report

Output (use `###` headings in the report):

> ### Summary
>
> The deploy-helper workflow has several structural problems: it does not encode its executable pipeline in the required first `dot` block, uses a non-standard ephemeral marker, and includes an unhelpful revision loop.
>
> ### Findings
>
> | Area | Status | Notes |
> |------|--------|-------|
> | Frontmatter | ⚠️ Warning | `ephemeral: true` is a non-standard field; temporary status should be represented by the `.gitignore` sentinel instead, and `deploy-helper` does not clearly follow the recommended `thing-process[-approach]` naming convention |
> | DOT pipeline structure | ❌ Fail | The first executable block is not a `dot` graph, so the workflow is incomplete for execution |
> | Workflow design quality | ❌ Fail | `Review -> Review` is a vague self-loop with no improving feedback path |
> | Agents and prompts | ⚠️ Warning | Prompts are too generic to guide meaningful execution |
> | Ephemeral conventions | ❌ Fail | Workflow appears intended to be temporary, but the required `.gitignore` sentinel is missing |
> | Completeness and clarity | ⚠️ Warning | The prose explains intent, but the executable structure is incomplete |
> | Consistency | ⚠️ Warning | Frontmatter, directory state, and body conventions do not align with workspace style |
>
> ### Suggestions
>
> 1. Move the executable pipeline into the first fenced `dot` block so the workflow can be validated and run
> 2. Remove `ephemeral: true` from frontmatter and, if the workflow is truly temporary, add a `.gitignore` file containing exactly `*`
> 3. Rename the workflow to something more process-oriented, such as `deployment-preparation` or `deployment-check-guided`, so the name communicates its end-to-end purpose rather than acting as a vague helper label
> 4. Replace the `Review -> Review` self-loop with a feedback path to a revisable node such as `Build` or `Draft`, so the loop can improve output quality
> 5. Rewrite node prompts to describe each step's local task and expected output more concretely

Input: "Review the paper-draft workflow with references"

Process:

1. Resolve to `.stencila/workflows/paper-draft/WORKFLOW.md`
2. Read the file and inspect the directory contents, including `references/brief.md` and `assets/template.md`
3. Confirm that the supporting files mentioned in the Markdown documentation actually exist and are relevant to the workflow
4. For the supporting files, verify existence and intended use without reproducing their full contents in the report
5. Compare the workflow's structure and documentation style against one or two existing workflows in `.stencila/workflows/`
6. Produce the report

Output (use `###` headings in the report):

> ### Summary
>
> The paper-draft workflow is well-documented and makes sensible use of supporting reference files. One improvement would make its supporting materials easier to understand and maintain.
>
> ### Findings
>
> | Area | Status | Notes |
> |------|--------|-------|
> | Frontmatter | ✅ Pass | Name, description, and goal are specific and aligned with the workflow |
> | DOT pipeline structure | ✅ Pass | The first `dot` block defines a clear research-to-draft pipeline |
> | Workflow design quality | ✅ Pass | Each stage has a distinct role and visible output |
> | Agents and prompts | ✅ Pass | Agent references look plausible, though existence is not fully verified in this review |
> | Ephemeral conventions | ✅ Pass | Workflow is permanent and does not use temporary markers |
> | Completeness and clarity | ⚠️ Warning | Supporting files exist and are referenced, but the documentation could explain more clearly how `template.md` influences the draft step |
> | Consistency | ✅ Pass | The workflow matches the style of other documentation-heavy workflows in the workspace |
>
> ### Suggestions
>
> 1. Add one sentence near the workflow documentation explaining when `assets/template.md` is used, so future reviewers and editors can understand its role without opening every supporting file

## Edge Cases

- **Workflow not found**: Report the error clearly and suggest checking the name or path. Prefer listing `.stencila/workflows/` directories directly; mention `stencila workflows list` only if that command is available in the current environment.
- **Multiple workflows requested**: Review each workflow separately with its own report section. Ask the user to confirm if reviewing all workflows is intended.
- **No body content**: Flag this as a failure — a workflow without a DOT pipeline is incomplete for execution unless the user explicitly asked for documentation only.
- **No `dot` block**: Flag this as a failure for an executable workflow. The first `dot` block is the executable pipeline source of truth.
- **Additional DOT blocks**: Treat only the first `dot` block as executable. If later DOT blocks could confuse the intended pipeline, flag that as a warning.
- **Unknown agents**: Use `list_agents` when agent selection matters so you can assess availability and metadata fit. If agents do not exist, distinguish between two cases: (a) the workflow appears to be designed top-down with intentionally planned agents — list them as outstanding dependencies to create, not as errors; (b) the names look like accidental placeholders or typos — flag them as warnings. The runtime accepts unresolved agent references (logging a warning and falling back to a default agent), so missing agents never block validation or execution.
- **Unknown child workflows**: Use `list_workflows` when workflow composition matters so you can assess availability and metadata fit. As with agents, distinguish intentional top-down forward references from accidental placeholders. List planned but not-yet-created child workflows as outstanding dependencies rather than flagging them as errors.
- **Ephemeral status unclear**: If the workflow seems temporary but lacks the `.gitignore` sentinel, flag that as a warning or failure depending on how strongly the workflow claims to be ephemeral.
- **Supporting files are large**: For files in `scripts/`, `references/`, or `assets/`, check that they exist and are referenced where appropriate, but do not reproduce their full contents in the report.
- **User asks to fix issues**: If the user asks you to apply suggestions, make the changes, then validate using the workflow directory path or `WORKFLOW.md` path before reporting completion; use name-based validation only when the name is unambiguous.

## Validation

When applying suggested improvements, validate the workflow before reporting completion:

```sh
# By directory path
stencila workflows validate .stencila/workflows/<workflow-name>

# By WORKFLOW.md path
stencila workflows validate .stencila/workflows/<workflow-name>/WORKFLOW.md

# By workflow name
stencila workflows validate <workflow-name>
```

Validation should pass before you report the changes as complete.

## Limitations

- This skill reviews the *structure, quality, and conventions* of a workflow definition. It does not execute the workflow or verify the runtime behavior of referenced agents
- DOT graph assessment is based on static review of the workflow definition, not full execution semantics
