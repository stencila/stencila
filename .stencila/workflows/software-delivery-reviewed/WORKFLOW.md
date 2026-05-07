---
name: software-delivery-reviewed
title: Software Delivery Reviewed Workflow
description: Execute a software design or delivery plan slice-by-slice using lighter implementation and parallel review cycles instead of full Red-Green-Refactor TDD
keywords:
  - software-delivery
  - implementation
  - slicing
  - review-driven
  - code-review
  - iterative
  - lighter-weight
  - plan-execution
goal-hint: Which design or delivery plan should be executed? Provide the plan content or reference the plan file path
when-to-use:
  - when a software design or delivery plan should be implemented incrementally but full TDD would be too slow or token intensive
  - when you want each slice implemented and then reviewed by the parallel code review workflow before moving on
  - when review-driven iteration is acceptable and strict test-first Red-Green-Refactor discipline is not required
when-not-to-use:
  - when the task explicitly requires test-driven development or test-first delivery (use software-delivery-tdd)
  - when no design or delivery plan exists yet and the work still needs specification or planning
  - when a one-pass implementation is sufficient and slice-by-slice review would be unnecessary overhead
---

This workflow executes a software design or delivery plan in successive slices, but uses a lighter review-driven loop than `software-delivery-tdd`. When given a delivery plan, it follows the plan's slices; when given a design specification, the slice-selection step derives practical execution units from the design's scope, acceptance criteria, and implementation notes. For each selected execution unit, the `software-implementor` agent implements or revises the slice, `code-review-parallel` performs a synthesized multi-model code review, and an assessment node decides whether the review contains actionable findings that should send the slice back for another implementation pass. When the review has no slice-blocking findings, the workflow returns to slice selection and continues until all slices are complete. A final completion step checks plan-level Definition of Done and a human review gate is kept only at the end.

The workflow intentionally does not create Red-phase tests, review those tests, or run a dedicated Refactor phase for every slice. The implementation agent may still run relevant checks and add or update tests when the plan or codebase requires them, but the control loop is implementation → synthesized code review → assessment. All long-running agent nodes use explicit `thread-id` values with full fidelity so repeated slice and revision passes can reuse session context while `max-session-turns` caps context growth.

```dot
digraph software_delivery_reviewed {
  node [max-session-turns="8"]

  Start -> SelectSlice

  SelectSlice [
    agent="software-slice-selector",
    prompt-ref="#select-slice-prompt",
    context-writable=true,
    fidelity="full",
    thread-id="slice-selector"
  ]
  SelectSlice -> Implement        [label="Continue"]
  SelectSlice -> CompleteDelivery [label="Done"]

  Implement [
    agent="software-implementor",
    prompt-ref="#implement-prompt",
    fidelity="full",
    thread-id="implementor",
    max-session-turns="5"
  ]
  Implement -> ReviewChanges

  ReviewChanges [
    workflow="code-review-parallel",
    label="Parallel code review",
    goal-ref="#review-changes-goal"
  ]
  ReviewChanges -> AssessReview

  AssessReview [
    agent="general",
    prompt-ref="#assess-review-prompt",
    context-writable=true,
    fidelity="full",
    thread-id="review-assessor"
  ]
  AssessReview -> Implement   [label="Revise"]
  AssessReview -> SelectSlice [label="Accept"]

  CompleteDelivery [
    agent="software-delivery-completer",
    prompt-ref="#complete-delivery-prompt",
    fidelity="full",
    thread-id="delivery-completer",
    max-session-turns="5"
  ]
  CompleteDelivery -> FinalHumanReview

  FinalHumanReview [interview-ref="#delivery-completion-review"]
  FinalHumanReview -> CommitCompletion [label="Accept and Commit"]
  FinalHumanReview -> End              [label="Accept"]
  FinalHumanReview -> CompleteDelivery [label="Closeout"]
  FinalHumanReview -> End              [label="Needs Plan Revision"]

  CommitCompletion [
    agent="general",
    prompt-ref="#commit-completion-prompt"
  ]
  CommitCompletion -> End
}
```

```markdown #select-slice-prompt
Mark the just-completed slice or slice batch, if any, and select the next unfinished execution unit from the design or delivery plan.

The design or delivery plan goal is: $goal

Note on "current_slice": this key serves a dual role. When entering SelectSlice after review acceptance it holds the most recently completed execution unit name. After SelectSlice stores a newly selected execution unit it holds the next unit to work on. On the first invocation it is empty. The selected unit may represent one plan slice or a combined batch of adjacent compatible slices.

**Step 1: read workflow state**

Use `workflow_get_context` once with `keys` to read:

- key "current_slice" — the most recently selected execution unit; treat it as just completed when re-entering after an accepted review, empty on first invocation
- key "completed_slices" — the list of previously completed slice names

**Step 2: delegate to the slice-selection skill**

Pass the just-completed slice or execution unit name, the completed slices list, and the plan goal to the slice-selection skill. The skill should:

- append the just-completed slice or underlying slices represented by a just-completed combined unit to the completed list, if provided
- read the design or delivery plan and identify the next unfinished execution unit, combining adjacent compatible slices when appropriate to avoid overly fine-grained iteration
- report the updated completed list, selected execution unit details, and whether more slices remain — or signal that all slices are complete

If the input is already a delivery plan, use its authored slices and sequencing as the primary source of truth. If the input is a design specification rather than a delivery plan, derive execution units from the design's requirements, acceptance criteria, affected components, risks, and implementation notes. Prefer coherent, independently reviewable units that can be implemented and assessed without needing a full planning workflow first; if the design is too ambiguous to derive safe execution units, report that clearly and route to "Done" only if no implementation work should proceed.

The selected execution unit details should include the slice name, included plan slices if this is a combined unit, scope, acceptance criteria, target packages/modules/directories, and any plan-specified or design-derived validation expectations.

**Step 3: clear stale slice-scoped context**

Use `workflow_set_context` once with `entries` to clear transient state from the previous slice so it cannot leak into the next one:

- key "review.feedback" — set to ""
- key "review.blocking_findings" — set to ""
- key "review.iteration_count" — set to 0

Do not clear "review.summary" here. It intentionally remains available to the completion step as the final accepted slice review summary after the last slice is marked complete. It will be overwritten by the next AssessReview node when another slice is implemented.

**Step 4: store the skill's outputs into workflow context**

Use `workflow_set_context` once with `entries` to store:

- key "completed_slices" — the updated completed slices list returned by the skill

If a slice or slice batch was selected, also store:

- key "current_slice" — the selected execution unit name or identifier
- key "slice.scope" — the scope description
- key "slice.acceptance_criteria" — the acceptance criteria
- key "slice.packages" — the packages, crates, modules, files, or directories involved
- key "slice.validation" — any plan-specified validation checks, or an empty string if none are specified

**Step 5: route**

If a slice or slice batch was selected, call `workflow_set_route` with label "Continue".
If all slices are complete, call `workflow_set_route` with label "Done".
```

```markdown #implement-prompt
Implement or revise the current slice of work.

The design or delivery plan goal is: $goal

**Step 1: read workflow state**

Use `workflow_get_context` once with `keys` to read:

- key "current_slice" — the selected execution unit name
- key "slice.scope" — what this execution unit covers
- key "slice.acceptance_criteria" — the acceptance criteria to satisfy
- key "slice.packages" — the packages, crates, modules, files, or directories involved
- key "slice.validation" — any plan-specified validation expectations
- key "review.feedback" — concise actionable review feedback from a previous pass, if any
- key "review.blocking_findings" — the blocking findings that caused a previous revision route, if any

Also use `workflow_get_output` when useful to inspect the immediately previous output. If this is a revision pass, the most useful feedback should be in the review context keys above, not embedded in this prompt.

**Step 2: implement or revise**

Make a focused implementation for the selected execution unit. If review feedback exists, address the concrete actionable findings first. Do not restart the slice from scratch unless the feedback clearly requires it.

Requirements:

- Satisfy the slice acceptance criteria
- Keep changes scoped to `slice.scope` and `slice.packages` unless a small supporting change elsewhere is clearly required
- Follow existing codebase conventions and architecture
- Add or update tests when the plan, codebase conventions, or changed behavior requires them, but do not create a full TDD Red-Green-Refactor cycle unless necessary
- Run the most relevant scoped checks you can reasonably identify, especially any checks listed in `slice.validation`
- Avoid broad cleanup or unrelated refactoring
- If you intentionally do not address a review finding, explain why in the implementation report

**Step 3: produce a review-ready implementation report**

End with a concise report containing:

- current slice name
- acceptance criteria addressed
- files changed
- notable design decisions
- tests or checks run, including command and result when applicable
- review findings addressed in this pass, if any
- known limitations or deferred items

The next node will pass this report to the parallel code review workflow.
```

```markdown #review-changes-goal
Review the implementation for the current delivery slice from this design or delivery plan:

$goal

Focus on correctness, acceptance criteria, regressions, security, maintainability, integration risks, and whether the slice is ready to mark complete. Review the changed files and the implementation report below.

Implementation report:
$last_output
```

```markdown #assess-review-prompt
Assess the synthesized parallel code review and decide whether the current slice needs another implementation pass.

**Step 1: read workflow state**

Use `workflow_get_context` once with `keys` to read:

- key "current_slice" — the selected execution unit name
- key "slice.scope" — what this execution unit covers
- key "slice.acceptance_criteria" — the acceptance criteria
- key "slice.packages" — the packages, crates, modules, files, or directories involved
- key "slice.validation" — any plan-specified validation expectations
- key "review.iteration_count" — the number of review assessment passes for this slice so far

Use `workflow_get_output` to retrieve the synthesized report from the `ReviewChanges` child workflow. If available, prefer the `ReviewChanges` node output over older node outputs.

**Step 2: classify findings**

Treat findings as slice-blocking when they are concrete, validated, and relevant to this execution unit, especially findings involving:

- correctness bugs
- unmet slice acceptance criteria
- regressions or broken behavior
- security issues
- data loss or corruption risks
- broken API or integration compatibility
- test, build, formatting, or validation failures that should be fixed before continuing
- substantial maintainability issues that make the slice unsafe or unusually costly to build on

Treat findings as non-blocking when they are only:

- style nits
- optional cleanup
- informational observations
- speculative issues not validated against actual code
- broader future work outside the current slice
- low-priority documentation polish that is not part of the slice acceptance criteria

If the synthesized review recommendation is "approve with changes", route to "Revise" only when the requested changes are concrete and should be addressed before marking this slice complete. If the remaining items are minor or explicitly deferrable, route to "Accept" and mention them in the summary.

**Iteration policy**

Use `review.iteration_count` to keep the loop bounded in practice:

- On the first two assessment passes for a slice, route to "Revise" for any concrete slice-blocking finding.
- On the third and later assessment passes, route to "Revise" only for critical correctness, security, data-loss, build/test failure, or clearly unmet acceptance-criteria issues.
- On the third and later assessment passes, route to "Accept" for medium/low maintainability issues, cleanup, style, documentation polish, or speculative findings, and document them as follow-up items in `review.summary` instead of continuing the loop.

If reviewers continue to find substantial non-critical work after repeated passes, prefer accepting the slice with documented follow-up or noting that the plan likely needs revision during final completion. Do not keep cycling indefinitely for non-critical improvements.

**Step 3: store concise review context**

Use `workflow_set_context` once with `entries` to store:

- key "review.summary" — a concise summary of the synthesized review and your decision
- key "review.blocking_findings" — a compact bullet list of findings that must be addressed before this slice can be accepted, or an empty string if none
- key "review.feedback" — actionable instructions for the implementor on the next pass, or an empty string if none
- key "review.iteration_count" — increment the previous iteration count by 1

Keep stored context compact. Do not paste the full synthesized review into context unless it is short.

**Step 4: route**

Call `workflow_set_route` with label "Revise" if there are slice-blocking findings that should be addressed by another implementation pass.
Call `workflow_set_route` with label "Accept" if there are no slice-blocking findings and the slice is ready to mark complete.
```

```markdown #complete-delivery-prompt
Complete the delivery after all execution slices have finished.

The design or delivery plan goal is: $goal

**Step 1: read workflow state**

Use `workflow_get_context` once with `keys` to read:

- key "current_slice" — the most recently completed execution unit
- key "completed_slices" — the completed plan slice identifiers
- key "completion.feedback" — any final-review follow-up notes from a prior closeout pass
- key "review.summary" — the final review assessment summary from the most recent slice, if useful

Use `workflow_get_output` to inspect the most recent workflow output when helpful.

**Step 2: inspect the plan, perform bounded closeout work, verify, and report**

Treat $goal as the design or delivery plan content, or as a reference to the plan file when that is how the workflow was invoked. Follow your agent instructions to inspect the plan, verify that all plan-level completion criteria and Definition of Done items have been addressed, perform minor closeout work if needed, run appropriate final checks, and produce a structured completion report.

If `completion.feedback` contains notes from a prior closeout pass, address those specific items in this iteration.

This is a bounded closeout pass. If substantial unfinished feature work remains, report that clearly rather than beginning a new large implementation cycle.
```

```yaml #delivery-completion-review
preamble: |
  All execution slices are complete. The delivery completer has checked the design or delivery plan's Definition of Done and other plan-level completion criteria, and has performed any minor closeout work it could.

  Please review the final completion report and the repository state before deciding how to finish.

  Use "Closeout" only for small remaining wrap-up items such as documentation touch-ups, verification gaps, generated artifacts, formatting or lint fixes, or other limited plan-completion tasks. If substantial feature work is still missing, the delivery plan likely needs to be extended or revisited outside this closeout loop.

questions:
  - header: Final Decision
    question: What should happen next for final delivery closeout?
    type: single-select
    options:
      - label: Accept and Commit
      - label: Accept
      - label: Closeout
      - label: Needs Plan Revision
    store: completion.decision

  - header: Closeout Notes
    question: What minor closeout items should be addressed?
    type: freeform
    store: completion.feedback
    show-if: "completion.decision == Closeout"

  - header: Plan Revision Notes
    question: What substantial missing work or plan changes are needed before delivery can continue?
    type: freeform
    store: completion.plan_revision_feedback
    show-if: "completion.decision == Needs Plan Revision"
```

```markdown #commit-completion-prompt
Commit the final delivery closeout changes.

Plan goal: $goal

**Step 1: review uncommitted changes**

Use the shell tool to inspect `git status` and `git diff --stat`.

**Step 2: stage delivery-related files**

Stage the files that are part of this reviewed delivery workflow. These may include production code, tests, documentation, generated artifacts, configuration, or other files changed while implementing and closing out the plan. Avoid staging unrelated changes.

**Step 3: commit**

Compose a commit message based on the final delivered work and the actual changes staged. Inspect the repository's recent commit history (`git log --oneline -20`) to infer the project's commit message conventions and follow them. Also check for any commit message instructions in the system prompt or prior context and apply those.

Run `git commit` with the composed message.

If any step fails (nothing to commit, git errors, etc.), report the issue but do not block workflow completion.
```
