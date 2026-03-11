---
name: workflow-creation-iterative
description: Create and iteratively refine a Stencila workflow using the workflow-creator and workflow-reviewer agents, route approved drafts through human review, and loop on requested revisions
goal: Produce an acceptable Stencila workflow definition for the requested purpose
keywords:
  - workflow
  - creation
  - review
  - iterative
  - human-in-the-loop
when-to-use:
  - when a new Stencila workflow needs to be drafted, reviewed, and refined before acceptance
  - when you want agent-based workflow authoring followed by explicit human approval cycles
when-not-to-use:
  - when you only need a one-pass workflow draft without review loops
  - when the task is to review an existing workflow without creating or refining it
---

```dot
digraph workflow_creation_iterative {
  Start -> Create

  Create [agent="workflow-creator", prompt-ref="#creator-prompt"]
  Create -> Review

  Review [agent="workflow-reviewer", prompt-ref="#reviewer-prompt"]
  Review -> HumanReview  [label="Accept", condition="context.last_output=yes"]
  Review -> Create       [label="Revise", condition="context.last_output!=yes"]

  HumanReview [interview-ref="#human-review-interview"]
  HumanReview -> End     [label="Accept"]
  HumanReview -> Create  [label="Revise"]
}
```

```text #creator-prompt
Create or update a Stencila workflow that helps users accomplish this underlying task: $goal

Interpret that as the end-user objective the workflow should support, not as an instruction to create another workflow. Ignore workflow-process phrasing such as iteration, review loops, or acceptance criteria unless it is genuinely part of the domain task.

If reviewer feedback is present, use it to revise the existing draft instead of starting over:
$last_output

If human feedback is present, incorporate it as well:
$human.feedback
```

```text #reviewer-prompt
Review the current workflow draft for the goal '$goal'. Ensure the workflow addresses the underlying user task rather than accidentally becoming a meta-workflow about creating workflows.

If the draft is acceptable, reply with ONLY yes in lowercase.
If the draft is not acceptable, reply with concrete revision feedback that the creator can use on the next pass.
```

```yaml #human-review-interview
preamble: |
  The workflow-reviewer agent has approved the current draft.
  Please review the workflow and decide whether to accept it or send it back for revision.

questions:
  - question: Is the workflow acceptable?
    header: Decision
    question_type: multiple_choice
    options:
      - label: Accept
      - label: Revise
    store: human.decision
    finish_if: Accept

  - question: What specific changes or improvements should be made?
    header: Revision Notes
    question_type: freeform
    store: human.feedback
```

The workflow first uses the `workflow-creator` agent to draft or revise the workflow, then uses a review step that emits a deterministic routing signal via `context.last_output`: `yes` means the draft is acceptable and any other output is treated as revision feedback and routed back to `Create`. The `Create` node consumes reviewer feedback from `$last_output` and any stored human revision notes from `$human.feedback`, so both automated and human guidance are available on iterative passes. After the reviewer approves, the workflow enters a structured human review interview. The decision question uses `finish_if: Accept` to end the interview immediately when the workflow is accepted, skipping the revision notes question. Choosing Revise continues the interview to collect feedback (stored as `human.feedback` for the next creator pass) and loops back to `Create`.
