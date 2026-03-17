---
name: software-slice-checking
description: Check whether a delivery plan has remaining slices after marking one as completed. Use when a slice of work has finished and the caller needs to know whether more slices remain. Given the just-finished slice name and the previously completed slices, appends the finished slice to the list, compares against the full plan, and reports whether more work remains or all slices are done.
keywords:
  - slice completion
  - completion check
  - delivery plan
  - remaining slices
  - plan progress
  - phased delivery
  - not slice selection
  - not plan creation
  - not test writing
  - not implementation
allowed-tools: read_file glob grep
---

## Overview

Check whether a delivery plan has remaining unfinished slices after marking the current slice as completed. This skill reads the plan, appends the just-finished slice to the completed list, and reports whether more work remains.

This skill does not select the next slice — use `software-slice-selection` for that. It only answers the question: "is there more work to do?"

## Required Inputs

| Input               | Required | Description                                                        |
|---------------------|----------|--------------------------------------------------------------------|
| Current slice       | Yes      | The slice name that was just completed                             |
| Completed slices    | No       | The list of previously completed slice names (empty if first)      |
| Plan name or goal   | No       | Which plan to read (defaults to the only or most recent plan)      |

When used standalone, these inputs come from the user or the agent's prompt. When used within a workflow, the workflow's stage prompt will specify how to obtain them.

## Outputs

| Output              | Description                                                                      |
|---------------------|----------------------------------------------------------------------------------|
| Updated completed slices | The completed slices list with the current slice appended                    |
| Status              | Whether more slices remain (more work) or all are done (no more work)            |

## Steps

1. Append the current slice to the completed list:
   - Take the provided completed slices list (may be a comma-separated string, a JSON array, or empty/absent)
   - Append the current slice name
   - Produce the updated completed slices list in the same format as the input, defaulting to comma-separated if starting fresh

2. Locate and read the delivery plan:
   - Use `glob` with pattern `.stencila/plans/*.md` to discover available plans
   - If the goal or prompt names a specific plan, locate it in the results
   - If only one plan exists, use it
   - Use `read_file` to load the full plan content

3. Parse the plan and compare:
   - Identify all phases and slices in the plan (same parsing rules as `software-slice-selection`)
   - Compare the updated completed slices list against the full set of slices in the plan
   - Determine whether any slices remain unfinished

4. Report the result:
   - **Updated completed slices**: the full list including the newly completed slice
   - **Status**: "more work remains" if unfinished slices exist, or "all work is complete" if every slice is in the completed list

## Examples

### Example 1: More slices remain

Current slice: "Phase 1 / Slice 2". Completed slices: "Phase 1 / Slice 1".

Action:
- Updated completed slices: "Phase 1 / Slice 1, Phase 1 / Slice 2"
- Plan has Phase 1 / Slice 3 remaining
- Status: more work remains

### Example 2: All slices done

Current slice: "Phase 2 / Slice 2". Completed slices: "Phase 1 / Slice 1, Phase 1 / Slice 2, Phase 2 / Slice 1".

Action:
- Updated completed slices: "Phase 1 / Slice 1, Phase 1 / Slice 2, Phase 2 / Slice 1, Phase 2 / Slice 2"
- Plan has no remaining slices
- Status: all work is complete

## Edge Cases

- **No delivery plan found**: Report the error clearly.
- **Current slice name does not match any plan slice**: Still append it. The plan may have been updated, or the name may use a slightly different format. Report the mismatch as a warning.
- **Current slice already in completed list**: Do not duplicate it. Report as normal.
- **Completed slices format varies**: Accept both comma-separated strings and JSON arrays. Write back in whichever format was read, defaulting to comma-separated if starting fresh.
