---
title: "Software Plan Iterative Workflow"
description: "Create and iteratively refine a software delivery plan using the `software-plan-creator` and `software-plan-reviewer` agents, with human review, optional commit, and revision loops until accepted"
keywords:
  - software-plan
  - delivery-plan
  - implementation-plan
  - review
  - iterative
  - human-in-the-loop
---

Create and iteratively refine a software delivery plan using the `software-plan-creator` and `software-plan-reviewer` agents, with human review, optional commit, and revision loops until accepted

**Keywords:** software-plan · delivery-plan · implementation-plan · review · iterative · human-in-the-loop

> [!tip] Usage
>
> To run this workflow, start your prompt with `~software-plan-iterative` followed by your goal, or select it with the `/workflow` command.

# When to use

- when a new software delivery plan needs to be drafted, reviewed, and refined before acceptance
- when you want agent-based delivery plan authoring followed by explicit human approval cycles

# When not to use

- when you only need a one-pass delivery plan draft without review loops
- when the task is to review an existing delivery plan without creating or refining it

# Configuration

| Property | Value |
| -------- | ----- |
| Goal | What feature or design spec do you want a delivery plan for? |
| Referenced agents | [`software-plan-creator`](/docs/agents/builtin/software/software-plan-creator/), [`software-plan-reviewer`](/docs/agents/builtin/software/software-plan-reviewer/), [`general`](/docs/agents/builtin/general/general/) |

# Pipeline

This workflow first uses the `software-plan-creator` agent to draft or revise the software delivery plan, then passes the draft to the `software-plan-reviewer` agent for review. The reviewer uses the `workflow_set_route` tool to choose between the `Accept` and `Revise` edge labels; when it chooses Revise its response text contains concrete revision feedback. The `Create` node uses the `workflow_get_output` tool to retrieve reviewer feedback and `workflow_get_context` with key `human.feedback` to retrieve human revision notes, so both automated and human guidance are available on iterative passes without bloating the prompt. After the reviewer accepts, the workflow enters a structured human review interview. Choosing Revise continues the interview to collect feedback (stored as `human.feedback` for the next creator pass) and loops back to `Create`. Choosing "Accept and Commit" routes through a Commit agent node that stages and commits the delivery plan artifact before ending the workflow.

```dot
digraph software_plan_iterative {
  Start -> Create

  Create [agent="software-plan-creator", prompt-ref="#creator-prompt"]
  Create -> Review

  Review [agent="software-plan-reviewer", prompt-ref="#reviewer-prompt"]
  Review -> HumanReview  [label="Accept"]
  Review -> Create       [label="Revise"]

  HumanReview [interview-ref="#human-review-interview"]
  HumanReview -> Commit  [label="Accept and Commit"]
  HumanReview -> End     [label="Accept"]
  HumanReview -> Create  [label="Revise"]

  Commit [agent="general", prompt-ref="#commit-prompt"]
  Commit -> End
}
```

## `creator-prompt`

Create or update a software delivery plan for the goal:

$goal

Before starting, use `workflow_get_output` to check for reviewer feedback from a previous iteration. If feedback is present, use it to revise the existing draft instead of starting over. If you disagree with a specific finding, you may provide a reasoned rebuttal instead of incorporating it.

Also use `workflow_get_context` with key "human.feedback" to check for human revision notes and incorporate those as well.

## `reviewer-prompt`

Review the current software delivery plan draft for the goal:

$goal

If the draft is acceptable, choose the Accept branch. If the draft needs changes, choose the Revise branch and provide specific revision feedback in your response.

## `human-review-interview`

```yaml #human-review-interview
preamble: |
  The software-plan-reviewer agent has approved the current draft.
  Please review the software delivery plan and decide whether to accept it or send it back for revision.

questions:
  - header: Decision
    question: Is the software delivery plan acceptable?
    type: single-select
    options:
      - label: Accept and Commit
      - label: Accept
      - label: Revise
    store: human.decision

  - header: Revision Notes
    question: What specific changes or improvements should be made?
    type: freeform
    store: human.feedback
    show-if: "human.decision == Revise"
```

## `commit-prompt`

Commit the delivery plan artifact.

Plan goal: $goal

**Step 1: stage changes**

Use the shell tool to review uncommitted changes with `git status` and `git diff --stat`.
Stage the delivery plan files. These are typically in `.stencila/plans/`. Use the goal
description as a guide, but include any other files that are clearly part of this planning
work. Avoid staging unrelated changes.

**Step 2: commit**

Compose a commit message based on the plan goal and the actual changes staged.
Inspect the repository's recent commit history (`git log --oneline -20`) to infer the
project's commit message conventions and follow them. Also check for any commit message
instructions in the system prompt or prior context and apply those.
Run `git commit` with the composed message.

If any step fails (nothing to commit, git errors, etc.), report the issue but do not block
the workflow — execution will continue regardless.

---

This page was generated from [`.stencila/workflows/software-plan-iterative/WORKFLOW.md`](https://github.com/stencila/stencila/blob/main/.stencila/workflows/software-plan-iterative/WORKFLOW.md).
