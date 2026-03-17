---
name: software-slice-selection
description: Select the next unfinished slice from a delivery plan based on phase ordering, dependency constraints, and a provided list of completed slices. Use when an agent needs to identify what to work on next from a phased delivery plan. Reads plans from .stencila/plans/, parses phase and slice structure, validates package references against the codebase, and reports the selected slice details or signals that all work is complete.
keywords:
  - slice selection
  - delivery plan
  - TDD slice
  - phased delivery
  - software delivery
  - plan reading
  - next slice
  - not plan creation
  - not code review
  - not test writing
  - not implementation
  - not completion check
allowed-tools: read_file glob grep
---

## Overview

Select the next unfinished slice from a software delivery plan stored in `.stencila/plans/`. Given a list of already-completed slices, identify the next slice to work on based on phase ordering and dependency constraints, validate its package references against the codebase, and report the slice details. If all slices are complete, report that instead.

This skill reads a plan file, parses its structure, and determines what comes next. It does not write code, create tests, modify the plan, or manage any external state — it reads the plan, evaluates progress, and reports its findings.

## Required Inputs

| Input               | Required | Description                                                        |
|---------------------|----------|--------------------------------------------------------------------|
| Completed slices    | No       | A list of slice names already finished (empty if starting fresh)   |
| Plan name or goal   | No       | Which plan to read (defaults to the only or most recent plan)      |

When used standalone, these inputs come from the user or the agent's prompt. When used within a workflow, the workflow's stage prompt will specify how to obtain them.

## Outputs

| Output              | Description                                                                      |
|---------------------|----------------------------------------------------------------------------------|
| Selected slice name | The slice name or identifier, or a signal that all slices are complete            |
| Scope               | A concise description of what the selected slice covers                          |
| Acceptance criteria | The criteria for the selected slice                                              |
| Packages            | The packages, crates, modules, or directories involved                           |
| Status              | Whether a slice was selected (more work) or all slices are complete (no more work)|

## Steps

1. Locate the delivery plan:
   - Use `glob` with pattern `.stencila/plans/*.md` to discover available plans
   - If the goal or prompt names a specific plan, locate it in the results
   - If only one plan exists, use it
   - Use `read_file` to load the full plan content
   - If no plans exist, report the error — there is nothing to execute

2. Parse the plan structure:
   - Identify all phases and their slices (TDD slices within phases, or tasks that serve as slices)
   - Record the name, scope, acceptance criteria, and relevant packages for each slice
   - Note dependency ordering — slices within a phase are ordered sequentially, and phases are ordered so earlier phases complete before later ones
   - A "slice" is typically a TDD slice defined in a phase's Testing section (e.g., "Slice 1: write failing test for X..."), but if the plan uses numbered tasks without explicit TDD slices, treat each task as a slice

3. Determine which slices are already completed:
   - Use the provided list of completed slices (may be a comma-separated string, a JSON array, or empty/absent meaning none completed)
   - Build a set of completed slice identifiers

4. Select the next unfinished slice:
   - Walk the plan in order: phases first (Phase 1 before Phase 2), then slices within each phase in order
   - Skip any slice whose identifier is in the completed set
   - Select the first unfinished slice
   - If all slices are completed, go to step 7

5. Validate package references against the codebase:
   - Use `glob` and `grep` to verify that the packages, crates, or directories mentioned in the slice actually exist in the workspace
   - If a package reference appears stale or incorrect, note it in the scope description so downstream work is aware
   - Do not block selection over a stale reference — it can be resolved during implementation

6. Report the selected slice:
   - Present the selected slice with its full details:
     - **Slice name**: the slice name or identifier (e.g., "Phase 1 / Slice 2")
     - **Scope**: a concise description of what this slice covers, derived from the plan
     - **Acceptance criteria**: the acceptance criteria, derived from the phase's exit criteria and the slice's specific test intent
     - **Packages**: the packages, crates, modules, or directories involved, derived from the plan and validated against the codebase
   - **Status**: more work remains
   - Stop

7. All slices complete:
   - If no unfinished slices remain, report that all plan slices have been completed
   - **Status**: all work is complete
   - Stop

## Slice Identification

Slices are identified using a combination of phase and slice number or name. Use a consistent naming scheme:

- If the plan has explicit slice names (e.g., "Token parsing validation"), use those prefixed with the phase: `Phase 1 / Token parsing validation`
- If the plan uses numbered slices (e.g., "Slice 1", "Slice 2"), use: `Phase 1 / Slice 1`
- If the plan uses numbered tasks without slices, use: `Phase 1 / Task 1`

Consistency matters because the names produced here may be used later to track completions — the name must be deterministic and reproducible for the same plan input.

## Dependency and Ordering Rules

- Complete all slices in Phase N before starting any slice in Phase N+1
- Within a phase, complete slices in their listed order (Slice 1 before Slice 2)
- If the plan explicitly marks a slice as depending on another, respect that dependency even if it crosses phases
- Never skip a slice — if a slice seems unnecessary, select it anyway and let the implementing agent determine the minimal work needed

## Examples

### Example 1: First slice selection with no prior completions

Plan has Phase 1 (3 slices) and Phase 2 (2 slices). No completed slices provided.

Action:
- Select Phase 1 / Slice 1
- Report: slice name = "Phase 1 / Slice 1", scope = "Define AuthToken and AuthError types", acceptance criteria = "AuthToken struct compiles with sub, exp, iat, roles fields; AuthError enum has MalformedToken variant", packages = "rust/auth"
- Status: more work remains

### Example 2: Mid-plan selection with some completions

Completed slices: "Phase 1 / Slice 1, Phase 1 / Slice 2, Phase 1 / Slice 3"

Action:
- All Phase 1 slices complete, move to Phase 2
- Select Phase 2 / Slice 1
- Report slice details
- Status: more work remains

### Example 3: All slices complete

Completed slices contains all slice identifiers from the plan.

Action:
- Report that all slices have been completed
- Status: all work is complete

## Edge Cases

- **No delivery plan found**: Report the error clearly. Do not fabricate a plan.
- **Plan has no identifiable slices or phases**: If the plan structure does not contain phases, TDD slices, or numbered tasks, treat the entire plan as a single slice. Select it on the first pass.
- **Ambiguous slice boundaries**: When the plan describes work narratively without clear slice demarcation, use phase-level tasks as slices. Prefer too few large slices over many tiny ones.
- **No completed slices provided**: Treat this as zero completions. Start from the first slice.
- **Completed slices format varies**: Accept both comma-separated strings ("Slice A, Slice B") and JSON arrays (`["Slice A", "Slice B"]`).
- **Package references are stale**: If a referenced package or directory no longer exists, note this in the scope description but still select the slice.
- **Plan was updated between selections**: If the plan file has changed since the last selection (new slices added, slices reordered), re-parse the full plan and reconcile against the completed slices. Previously completed slices remain completed; new slices are eligible for selection in order.
