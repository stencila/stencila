---
name: software-slice-selection
description: Select the next unfinished slice from a delivery plan, store its metadata in workflow context, track completed slices, and route the workflow accordingly. Use when a workflow needs to pick the next TDD slice from a phased delivery plan, mark slices as completed, or determine whether all slices are done. Reads plans from .stencila/plans/, manages completion state via workflow context, and signals Continue or Done routing.
keywords:
  - slice selection
  - delivery plan
  - TDD slice
  - workflow context
  - slice routing
  - completion tracking
  - phased delivery
  - red green refactor
  - software delivery
  - plan execution
  - not plan creation
  - not code review
  - not test writing
  - not implementation
allowed-tools: read_file glob grep
---

## Overview

Select the next unfinished slice from a software delivery plan stored in `.stencila/plans/`, store its metadata in workflow context for downstream agents, track which slices have been completed, and route the workflow to either continue with the next slice or signal that all work is done.

This skill is used by agents operating within workflows like `software-delivery-tdd`, where a slice selector agent runs at the start and after each completed slice to manage the iteration loop. The skill does not write code, create tests, or modify the plan — it reads the plan, determines what to do next, and stores the relevant context.

## Context Keys

These are the workflow context keys this skill reads and writes:

| Key                        | Direction | Type         | Description                                                        |
|----------------------------|-----------|--------------|--------------------------------------------------------------------|
| `completed_slices`         | Read/Write| JSON string  | Comma-separated or JSON array of slice names already finished      |
| `current_slice`            | Write     | String       | Name or identifier of the selected slice                           |
| `slice.scope`              | Write     | String       | Concise description of what the slice covers                       |
| `slice.acceptance_criteria`| Write     | String       | Acceptance criteria for the slice                                  |
| `slice.packages`           | Write     | String       | Packages, crates, modules, or directories involved                 |

## Route Labels

| Label      | When to use                                         |
|------------|-----------------------------------------------------|
| `Continue` | A slice was selected and its context has been stored |
| `Done`     | All slices in the plan have been completed           |

## Steps

### Slice Selection Mode (initial selection)

1. Locate the delivery plan:
   - Use `glob` with pattern `.stencila/plans/*.md` to discover available plans
   - If the workflow goal names a specific plan, locate it in the results
   - If only one plan exists, use it
   - Use `read_file` to load the full plan content
   - If no plans exist, report the error and choose the `Done` route — there is nothing to execute

2. Parse the plan structure:
   - Identify all phases and their slices (TDD slices within phases, or tasks that serve as slices)
   - Record the name, scope, acceptance criteria, and relevant packages for each slice
   - Note dependency ordering — slices within a phase are ordered sequentially, and phases are ordered so earlier phases complete before later ones
   - A "slice" is typically a TDD slice defined in a phase's Testing section (e.g., "Slice 1: write failing test for X..."), but if the plan uses numbered tasks without explicit TDD slices, treat each task as a slice

3. Check for previously completed slices:
   - Call `workflow_get_context` with key `completed_slices`
   - Parse the result — it may be a comma-separated string, a JSON array, or absent (meaning no slices completed yet)
   - Build a set of completed slice identifiers

4. Select the next unfinished slice:
   - Walk the plan in order: phases first (Phase 1 before Phase 2), then slices within each phase in order
   - Skip any slice whose identifier is in the completed set
   - Select the first unfinished slice
   - If all slices are completed, go to step 7

5. Validate package references against the codebase:
   - Use `glob` and `grep` to verify that the packages, crates, or directories mentioned in the slice actually exist in the workspace
   - If a package reference appears stale or incorrect, note it in the slice scope description so downstream agents are aware
   - Do not block selection over a stale reference — downstream agents can resolve it

6. Store the slice metadata and route:
   - Call `workflow_set_context` for each key:
     - `current_slice` — the slice name or identifier (e.g., "Phase 1 / Slice 2" or "phase-1-slice-2")
     - `slice.scope` — a concise description of what this slice covers, derived from the plan
     - `slice.acceptance_criteria` — the acceptance criteria, derived from the phase's exit criteria and the slice's specific test intent
     - `slice.packages` — the packages, crates, modules, or directories involved, derived from the plan and validated against the codebase
   - Present the selected slice to the user with its details
   - Call `workflow_set_route` with label `Continue`
   - Stop

7. All slices complete:
   - If no unfinished slices remain, report that all plan slices have been completed
   - Call `workflow_set_route` with label `Done`
   - Stop

### Completion Check Mode (after a slice is finished)

This mode is used at the `CheckRemaining` node after a slice passes human review.

1. Read the current slice and completed slices:
   - Call `workflow_get_context` with key `current_slice` to get the slice that was just finished
   - Call `workflow_get_context` with key `completed_slices` to get previously completed slices

2. Mark the current slice as completed:
   - Append the current slice name to the completed slices list
   - Call `workflow_set_context` with key `completed_slices` and the updated list (as a comma-separated string or JSON array, consistent with the existing format)

3. Check for remaining slices:
   - Locate and read the delivery plan (same as step 1-2 in Selection Mode)
   - Compare the updated completed slices against the full plan
   - If unfinished slices remain, call `workflow_set_route` with label `Continue`
   - If all slices are done, call `workflow_set_route` with label `Done`

## Slice Identification

Slices are identified using a combination of phase and slice number or name. Use a consistent naming scheme throughout the workflow run:

- If the plan has explicit slice names (e.g., "Token parsing validation"), use those prefixed with the phase: `Phase 1 / Token parsing validation`
- If the plan uses numbered slices (e.g., "Slice 1", "Slice 2"), use: `Phase 1 / Slice 1`
- If the plan uses numbered tasks without slices, use: `Phase 1 / Task 1`

Consistency matters because `completed_slices` is a string comparison — the name stored at selection time must match the name used at completion check time.

## Dependency and Ordering Rules

- Complete all slices in Phase N before starting any slice in Phase N+1
- Within a phase, complete slices in their listed order (Slice 1 before Slice 2)
- If the plan explicitly marks a slice as depending on another, respect that dependency even if it crosses phases
- Never skip a slice — if a slice seems unnecessary, select it anyway and let the downstream agents determine the minimal work needed

## Examples

### Example 1: First slice selection with no prior completions

Plan has Phase 1 (3 slices) and Phase 2 (2 slices). No `completed_slices` in context.

Action:
- Select Phase 1 / Slice 1
- Store: `current_slice` = "Phase 1 / Slice 1", `slice.scope` = "Define AuthToken and AuthError types", `slice.acceptance_criteria` = "AuthToken struct compiles with sub, exp, iat, roles fields; AuthError enum has MalformedToken variant", `slice.packages` = "rust/auth"
- Route: `Continue`

### Example 2: Mid-plan selection with some completions

`completed_slices` = "Phase 1 / Slice 1, Phase 1 / Slice 2, Phase 1 / Slice 3"

Action:
- All Phase 1 slices complete, move to Phase 2
- Select Phase 2 / Slice 1
- Store context keys for the new slice
- Route: `Continue`

### Example 3: All slices complete

`completed_slices` contains all slice identifiers from the plan.

Action:
- Report that all slices have been completed
- Route: `Done`

### Example 4: Completion check after human approval

`current_slice` = "Phase 1 / Slice 2", `completed_slices` = "Phase 1 / Slice 1"

Action:
- Append "Phase 1 / Slice 2" to completed slices
- Store: `completed_slices` = "Phase 1 / Slice 1, Phase 1 / Slice 2"
- Plan has Phase 1 / Slice 3 remaining
- Route: `Continue`

## Edge Cases

- **No delivery plan found**: Report the error clearly. Do not fabricate a plan. Route `Done` to prevent the workflow from looping indefinitely.
- **Plan has no identifiable slices or phases**: If the plan structure does not contain phases, TDD slices, or numbered tasks, treat the entire plan as a single slice. Select it on the first pass, mark it complete on the check pass.
- **Ambiguous slice boundaries**: When the plan describes work narratively without clear slice demarcation, use phase-level tasks as slices. Prefer too few large slices over many tiny ones — downstream agents can scope their own work within a slice.
- **completed_slices key missing or empty**: Treat this as zero completions. Start from the first slice.
- **completed_slices format varies**: Accept both comma-separated strings ("Slice A, Slice B") and JSON arrays (`["Slice A", "Slice B"]`). Write back in whichever format was read, defaulting to comma-separated if starting fresh.
- **Package references are stale**: If a referenced package or directory no longer exists (perhaps renamed or removed in a prior slice), note this in `slice.scope` but still select the slice. The implementing agent will resolve the discrepancy.
- **Plan was updated between slices**: If the plan file has changed since the last selection (new slices added, slices reordered), re-parse the full plan and reconcile against `completed_slices`. Previously completed slices remain completed; new slices are eligible for selection in order.
