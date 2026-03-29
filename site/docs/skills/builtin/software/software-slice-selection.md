---
title: "Software Slice Selection"
description: "Mark the just-completed slice or slice batch (if any), update the completed slices list, select the next unfinished execution unit from a delivery plan based on phase ordering and dependency constraints, and report whether more slices remain. Determines slice exhaustion, not full plan completion — does not verify plan-level Definition of Done items. Combines completion tracking with next-work selection in a single step. Reads plans from .stencila/plans/, parses phase and slice structure, validates package references against the codebase, and can normalize overly narrow plan slices by combining adjacent compatible slices into one execution unit."
keywords:
  - slice selection
  - delivery plan
  - TDD slice
  - phased delivery
  - software delivery
  - plan reading
  - next slice
  - slice completion
  - completion check
  - remaining slices
  - plan progress
  - not plan creation
  - not code review
  - not test writing
  - not implementation
---

Mark the just-completed slice or slice batch (if any), update the completed slices list, select the next unfinished execution unit from a delivery plan based on phase ordering and dependency constraints, and report whether more slices remain. Determines slice exhaustion, not full plan completion — does not verify plan-level Definition of Done items. Combines completion tracking with next-work selection in a single step. Reads plans from .stencila/plans/, parses phase and slice structure, validates package references against the codebase, and can normalize overly narrow plan slices by combining adjacent compatible slices into one execution unit.

**Keywords:** slice selection · delivery plan · TDD slice · phased delivery · software delivery · plan reading · next slice · slice completion · completion check · remaining slices · plan progress · not plan creation · not code review · not test writing · not implementation

# Configuration

| Property | Value |
| -------- | ----- |
| Allowed tools | `read_file`, `glob`, `grep` |

# Instructions

## Overview

Mark the just-completed slice or slice batch (if any), update the completed slices list, and select the next unfinished execution unit from a software delivery plan stored in `.stencila/plans/`. Given a just-finished slice name and a list of already-completed slices, append the finished slice or slices, identify the next unit of work to execute based on phase ordering and dependency constraints, validate its package references against the codebase, and report the unit details along with the updated completed list. If all slices are complete, report that no slices remain.

This skill determines **slice exhaustion** — whether there are more execution slices to select — not full plan completion. Many plans include a `## Definition of Done` section with plan-level closure checks (e.g., all tests passing, linting clean, generated types updated, documentation complete). This skill does not evaluate those checks. When all slices have been selected and completed, the skill reports that no slices remain, but the caller must verify Definition of Done items separately before treating the plan as finished.

This skill reads a plan file, parses its structure, tracks completions, and determines what comes next. It does not write code, create tests, modify the plan, or manage any external state beyond reporting its findings. It also acts as a normalizer of planning granularity: plan-authored slices are advisory, and the selected execution unit may consist of one slice or several adjacent compatible slices when that would reduce workflow overhead without creating an unmanageably broad TDD loop.

## Required Inputs

| Input               | Required | Description                                                        |
|---------------------|----------|--------------------------------------------------------------------|
| Just-completed slice | No      | The slice name that was just completed (empty on first invocation) |
| Completed slices    | No       | A list of slice names already finished (empty if starting fresh)   |
| Plan name or goal   | No       | Which plan to read (defaults to the only or most recent plan)      |

When used standalone, these inputs come from the user or the agent's prompt. When used within a workflow, the workflow's stage prompt will specify how to obtain them.

## Outputs

| Output                   | Description                                                                      |
|--------------------------|----------------------------------------------------------------------------------|
| Updated completed slices | The completed slices list with the just-finished slice or slices appended (if provided) |
| Selected slice name      | The selected execution unit name or identifier, or a signal that no slices remain |
| Included slices          | The underlying plan slice identifiers covered by the selected execution unit |
| Scope                    | A concise description of what the selected execution unit covers |
| Acceptance criteria      | The criteria for the selected execution unit |
| Packages                 | The packages, crates, modules, or directories involved |
| Status                   | Whether an execution unit was selected (more slices remain) or no slices remain (slice exhaustion, but not necessarily plan-level Definition of Done)|

## Steps

1. Mark the just-completed slice or slice batch (if provided):
   - If a just-completed slice name is provided and non-empty:
     - Take the provided completed slices list (may be a comma-separated string, a JSON array, or empty/absent)
      - Determine whether the just-completed name refers to a single slice or to a previously selected combined execution unit by re-parsing the plan and matching against the same selection rules used below
      - If it matches a combined execution unit, append each underlying plan slice identifier that is not already in the list
      - Otherwise, if the just-completed slice is not already in the list, append it
     - Produce the updated completed slices list in the same format as the input, defaulting to comma-separated if starting fresh
   - If no just-completed slice is provided (first invocation), use the completed slices list as-is

2. Locate the delivery plan:
   - Use `glob` with pattern `.stencila/plans/*.md` to discover available plans
   - If the goal or prompt names a specific plan, locate it in the results
   - If only one plan exists, use it
   - Use `read_file` to load the full plan content
   - If no plans exist, report the error — there is nothing to execute

3. Parse the plan structure:
   - Identify all phases and their slices (TDD slices within phases, or tasks that serve as slices)
   - Record the name, scope, acceptance criteria, and relevant packages for each slice
   - Note dependency ordering — slices within a phase are ordered sequentially, and phases are ordered so earlier phases complete before later ones
   - A "slice" is typically a TDD slice defined in a phase's Testing section (e.g., "Slice 1: write failing test for X..."), but if the plan uses numbered tasks without explicit TDD slices, treat each task as a slice
   - Note whether the plan contains a `## Definition of Done` section. Record its presence for use in the final reporting step, but do not convert Definition of Done items into slices

4. Determine which slices are already completed:
   - Use the updated completed slices list from step 1 (may be a comma-separated string, a JSON array, or empty/absent meaning none completed)
   - Build a set of completed slice identifiers

5. Select the next unfinished execution unit:
   - Walk the plan in order: phases first (Phase 1 before Phase 2), then slices within each phase in order
   - Skip any slice whose identifier is in the completed set
   - Start with the first unfinished slice as the candidate execution unit
   - Consider combining the candidate with following adjacent unfinished slices in the same phase when all of the following are true:
     - the slices are contiguous in the plan and there is no unfinished slice between them
     - there is no explicit dependency boundary that requires the earlier slice to be validated independently
     - they touch the same package, crate, module, or tightly related directory area
     - they are small and closely related enough that one Red-Green-Refactor cycle remains tractable
     - combining them is likely to reduce workflow overhead materially relative to doing them one-by-one
   - Stop combining when adding another slice would make the execution unit too broad, too risky, cross a package or subsystem boundary, or obscure a meaningful review checkpoint
    - When synthesizing scope and acceptance criteria for the selected unit, preserve documentation or other non-test deliverables from the plan, but do not frame them as implying mandatory unit/integration tests; downstream stages may satisfy them through implementation, review notes, or explicit existence checks instead of Red-phase tests
   - If all slices are completed, go to step 8

6. Validate package references against the codebase:
   - Use `glob` and `grep` to verify that the packages, crates, or directories mentioned in the slice actually exist in the workspace
   - If a package reference appears stale or incorrect, note it in the scope description so downstream work is aware
   - Do not block selection over a stale reference — it can be resolved during implementation

7. Report the selected execution unit:
   - Present the selected unit with its full details:
     - **Updated completed slices**: the full list including the newly completed slice (if any)
      - **Slice name**: the execution unit name or identifier (e.g., `Phase 1 / Slice 2` for a single slice, or `Phase 1 / Slices 2-3` for a combined unit)
      - **Included slices**: the underlying plan slice identifiers included in the execution unit
      - **Scope**: a concise description of what this execution unit covers, derived from the plan
      - **Acceptance criteria**: the acceptance criteria for the unit, combining the included slices' intent while keeping the scope coherent
      - **Packages**: the packages, crates, modules, or directories involved, derived from the plan and validated against the codebase
   - **Status**: more work remains
   - Stop

8. All slices complete:
   - If no unfinished slices remain, report:
     - **Updated completed slices**: the full list
     - That all execution slices have been completed and no slices remain for selection
     - If the plan contains a `## Definition of Done` section, include a note: "All execution slices are complete. The plan includes a Definition of Done section — verify those plan-level completion checks separately before treating the plan as finished."
     - If the plan does not contain a `## Definition of Done` section, include a note: "All execution slices are complete. Verify any plan-level completion criteria separately before treating the plan as finished."
   - **Status**: all slices are complete
   - Stop

## Output Format

Always end your response with a structured report using exactly these labeled fields so the caller can reliably extract each value:

```
Updated completed slices: <comma-separated list, or "(none)" if empty>
Slice name: <selected slice identifier, or "ALL SLICES COMPLETE" if no slices remain>
Included slices: <comma-separated list of exact plan slice identifiers included in the selected execution unit; use "(none)" only when Slice name is "ALL SLICES COMPLETE">
Scope: <concise scope description>
Acceptance criteria: <the criteria for this slice>
Packages: <comma-separated list of packages, crates, or directories>
Status: <"more work remains" or "all slices are complete">
```

When all slices are complete, omit the Scope, Acceptance criteria, and Packages fields and set `Included slices` to `(none)`. Include a Definition of Done note if the plan has one:

```
Updated completed slices: Phase 1 / Slice 1, Phase 1 / Slice 2
Slice name: ALL SLICES COMPLETE
Included slices: (none)
Status: all slices are complete
Note: All execution slices are complete. The plan includes a Definition of Done section — verify those plan-level completion checks separately before treating the plan as finished.
```

Narrative explanation or warnings (e.g. stale package references) may precede the structured block, but the block itself must always appear last and use the exact field labels above.

When the selected execution unit consists of a single plan slice, the `Included slices` field should contain just that one slice identifier. When the selected execution unit combines several adjacent plan slices, the `Slice name` should be a concise synthesized label and `Included slices` must enumerate the exact underlying plan slices in execution order. Do not use ranges, abbreviations, or paraphrases in `Included slices`; use the exact deterministic slice identifiers that should be added to `completed_slices` once the unit is accepted.

## Slice Identification

Slices are identified using a combination of phase and slice number or name. Use a consistent naming scheme:

- If the plan has explicit slice names (e.g., "Token parsing validation"), use those prefixed with the phase: `Phase 1 / Token parsing validation`
- If the plan uses numbered slices (e.g., "Slice 1", "Slice 2"), use: `Phase 1 / Slice 1`
- If the plan uses numbered tasks without slices, use: `Phase 1 / Task 1`

Consistency matters because the names produced here may be used later to track completions — the name must be deterministic and reproducible for the same plan input.

## Dependency and Ordering Rules

- Complete all slices in Phase N before starting any slice in Phase N+1
- Within a phase, preserve slice ordering. You may combine adjacent compatible slices into one execution unit, but never reorder them
- If the plan explicitly marks a slice as depending on another, respect that dependency even if it crosses phases
- Never skip a slice. If a slice seems unnecessary, still include it either as its own execution unit or within a combined adjacent unit

## Granularity Normalization Rules

- Treat plan-authored slices as advisory execution hints, not mandatory one-iteration boundaries
- Prefer one coherent execution unit per workflow iteration, not necessarily one literal plan slice
- Prefer combining slices only when they are adjacent, small, and tightly related
- Do not combine across phase boundaries
- Do not combine across explicit dependency boundaries that call for independent validation
- Do not combine slices that span unrelated packages, crates, modules, or subsystems unless the plan clearly frames them as one inseparable change
- Bias toward fewer, more meaningful iterations when the plan is overly granular, but keep each execution unit small enough for one tractable Red-Green-Refactor cycle and one human review
- When uncertain, prefer a modest combination of two or a few closely related slices over a large batch

## Examples

### Example 1: First slice selection with no prior completions

Plan has Phase 1 (3 slices) and Phase 2 (2 slices). No just-completed slice. No completed slices provided.

Action:
- No completion to record (first invocation)
- Updated completed slices: (empty)
- Select Phase 1 / Slice 1
- Report: slice name = "Phase 1 / Slice 1", included slices = "Phase 1 / Slice 1", scope = "Define AuthToken and AuthError types", acceptance criteria = "AuthToken struct compiles with sub, exp, iat, roles fields; AuthError enum has MalformedToken variant", packages = "rust/auth"
- Status: more work remains

### Example 2: Mark completion and select next slice

Just-completed slice: "Phase 1 / Slice 2". Completed slices: "Phase 1 / Slice 1".

Action:
- Append "Phase 1 / Slice 2" to completed list
- Updated completed slices: "Phase 1 / Slice 1, Phase 1 / Slice 2"
- Select Phase 1 / Slice 3
- Report slice details including included slices
- Status: more work remains

### Example 3: Combine adjacent narrow slices into one execution unit

Plan has `Phase 1 / Slice 1` = add parser tests, `Phase 1 / Slice 2` = add parser implementation, and `Phase 1 / Slice 3` = add parser error formatting. Slices 2 and 3 both affect the same module and are both small.

Action:
- Completed slices: "Phase 1 / Slice 1"
- Candidate next slice is `Phase 1 / Slice 2`
- Combine with adjacent `Phase 1 / Slice 3` because both are small, contiguous, same-package changes with no meaningful review boundary between them
- Report: slice name = "Phase 1 / Slices 2-3", included slices = "Phase 1 / Slice 2, Phase 1 / Slice 3", scope = "Implement parser behavior and error formatting", acceptance criteria = "Parser behavior matches tests and error formatting is emitted as specified", packages = "rust/parser"
- Status: more work remains

### Example 4: Mark completion crossing a phase boundary

Just-completed slice: "Phase 1 / Slice 3". Completed slices: "Phase 1 / Slice 1, Phase 1 / Slice 2".

Action:
- Append "Phase 1 / Slice 3" to completed list
- Updated completed slices: "Phase 1 / Slice 1, Phase 1 / Slice 2, Phase 1 / Slice 3"
- All Phase 1 slices complete, move to Phase 2
- Select Phase 2 / Slice 1
- Report slice details
- Status: more work remains

### Example 5: Mark final completion — no slices remain

Just-completed slice: "Phase 2 / Slice 2". Completed slices: "Phase 1 / Slice 1, Phase 1 / Slice 2, Phase 1 / Slice 3, Phase 2 / Slice 1". The plan contains a `## Definition of Done` section.

Action:
- Append "Phase 2 / Slice 2" to completed list
- Updated completed slices: "Phase 1 / Slice 1, Phase 1 / Slice 2, Phase 1 / Slice 3, Phase 2 / Slice 1, Phase 2 / Slice 2"
- No remaining slices
- Note: All execution slices are complete. The plan includes a Definition of Done section — verify those plan-level completion checks separately before treating the plan as finished.
- Status: all slices are complete

## Edge Cases

- **No delivery plan found**: Report the error clearly. Do not fabricate a plan.
- **Plan has no identifiable slices or phases**: If the plan structure does not contain phases, TDD slices, or numbered tasks, treat the entire plan as a single slice. Select it on the first pass.
- **Ambiguous slice boundaries**: When the plan describes work narratively without clear slice demarcation, use phase-level tasks as slices. Prefer too few coherent slices over many tiny ones.
- **No completed slices provided**: Treat this as zero completions. Start from the first slice.
- **Completed slices format varies**: Accept both comma-separated strings ("Slice A, Slice B") and JSON arrays (`["Slice A", "Slice B"]`). Write back in whichever format was read, defaulting to comma-separated if starting fresh.
- **Just-completed slice already in completed list**: Do not duplicate it. Proceed with selection as normal.
- **Just-completed slice name does not match any plan slice**: Still append it. The plan may have been updated, or the name may use a slightly different format. Report the mismatch as a warning.
- **Package references are stale**: If a referenced package or directory no longer exists, note this in the scope description but still select the slice.
- **Plan was updated between selections**: If the plan file has changed since the last selection (new slices added, slices reordered), re-parse the full plan and reconcile against the completed slices. Previously completed slices remain completed; new slices are eligible for selection in order.
- **Plan is overly granular**: Combine adjacent compatible slices into one execution unit when doing so will materially reduce workflow overhead without making the TDD cycle too broad.

---

This page was generated from [`.stencila/skills/software-slice-selection/SKILL.md`](https://github.com/stencila/stencila/blob/main/.stencila/skills/software-slice-selection/SKILL.md).
