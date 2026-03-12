---
name: software-design-iterative
description: Create and iteratively refine a software design specification using the software-design-creator and software-design-reviewer agents, then continue through human review until accepted
goal-hint: What feature or system do you want to design?
keywords:
  - software-design
  - design-specification
  - review
  - iterative
  - human-in-the-loop
when-to-use:
  - when a new software design specification needs to be drafted, reviewed, and refined before acceptance
  - when you want agent-based software design authoring followed by explicit human approval cycles
when-not-to-use:
  - when you only need a one-pass software design draft without review loops
  - when the task is to review an existing design specification without creating or refining it
---

```dot
digraph software_design_iterative {
  Start -> Create

  Create [agent="software-design-creator", prompt-ref="#creator-prompt"]
  Create -> Review

  Review [agent="software-design-reviewer", prompt-ref="#reviewer-prompt"]
  Review -> HumanReview  [label="Accept"]
  Review -> Create       [label="Revise"]

  HumanReview [interview-ref="#human-review-interview"]
  HumanReview -> End     [label="Accept"]
  HumanReview -> Create  [label="Revise"]
}
```

```text #creator-prompt
Create or update a software design specification that helps users accomplish this underlying task: $goal

Interpret that as the end-user objective the software design specification should support, not as an instruction to create another workflow. Ignore workflow-process phrasing such as iteration, review loops, or acceptance criteria unless it is genuinely part of the domain task.

If reviewer feedback is present, use it to revise the existing draft instead of starting over:
$last_output

If human feedback is present, incorporate it as well:
$human.feedback
```

```text #reviewer-prompt
Review the current software design draft for the goal '$goal'. Ensure the design addresses the underlying user task rather than accidentally becoming a meta-design about creating workflows.

If the draft is acceptable, choose the Accept branch. If the draft needs changes, choose the Revise branch and provide specific revision feedback in your response.
```

```yaml #human-review-interview
preamble: |
  The software-design-reviewer agent has approved the current draft.
  Please review the software design specification and decide whether to accept it or send it back for revision.

questions:
  - header: Decision
    question: Is the software design specification acceptable?
    type: single-select
    options:
      - label: Accept
      - label: Revise
    store: human.decision
    finish-if: Accept

  - header: Revision Notes
    question: What specific changes or improvements should be made?
    type: freeform
    store: human.feedback
```

The workflow first uses the `software-design-creator` agent to draft or revise the software design specification, then passes the draft to the `software-design-reviewer` agent for review. The reviewer uses the `set_preferred_label` tool (provided automatically) to choose between the `Accept` and `Revise` edge labels; when it chooses Revise its response text contains concrete revision feedback that is routed back to `Create` via `$last_output`. The `Create` node consumes reviewer feedback from `$last_output` and any stored human revision notes from `$human.feedback`, so both automated and human guidance are available on iterative passes. After the reviewer accepts, the workflow enters a structured human review interview. The decision question uses `finish-if: Accept` to end the interview immediately when the software design specification is accepted, skipping the revision notes question. Choosing Revise continues the interview to collect feedback (stored as `human.feedback` for the next creator pass) and loops back to `Create`.
