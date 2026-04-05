---
title: "Theme Creation Iterative Workflow"
description: "Create and iteratively refine a Stencila theme using the `theme-creator` and `theme-reviewer` agents, route approved drafts through human review with optional commit, and loop on requested revisions"
keywords:
  - theme
  - creation
  - review
  - iterative
  - human-in-the-loop
  - theme.css
  - design tokens
  - css
---

Create and iteratively refine a Stencila theme using the `theme-creator` and `theme-reviewer` agents, route approved drafts through human review with optional commit, and loop on requested revisions

**Keywords:** theme · creation · review · iterative · human-in-the-loop · theme.css · design tokens · css

> [!tip] Usage
>
> To run this workflow, start your prompt with `~theme-creation-iterative` followed by your goal, or select it with the `/workflow` command.

# When to use

- when a new Stencila theme needs to be drafted, reviewed, and refined before acceptance
- when you want agent-based theme authoring followed by explicit human approval cycles
- when creating a document theme, site theme, plot theme, or combined theme with iterative refinement

# When not to use

- when you only need a one-pass theme draft without review loops
- when the task is to review an existing theme without creating or refining it
- when modifying a theme with a single targeted change that does not need iterative review

# Configuration

| Property | Value |
| -------- | ----- |
| Goal | Describe the theme you want to create — what style, mood, or design direction should it have? |
| Referenced agents | [`theme-creator`](/docs/agents/builtin/themes/theme-creator/), [`theme-reviewer`](/docs/agents/builtin/themes/theme-reviewer/), [`general`](/docs/agents/builtin/general/general/) |

# Pipeline

This workflow first uses the `theme-creator` agent to draft or revise a theme CSS file, then passes the draft to the `theme-reviewer` agent for review. The reviewer uses the `workflow_set_route` tool to choose between the `Accept` and `Revise` edge labels; when it chooses Revise its response text contains concrete revision feedback.

The `Create` node uses the `workflow_get_output` tool to retrieve reviewer feedback and `workflow_get_context` with key `human.feedback` to retrieve human revision notes, so both automated and human guidance are available on iterative passes without bloating the prompt. After the reviewer accepts, the workflow enters a structured human review interview. Choosing Revise continues the interview to collect feedback (stored as `human.feedback` for the next creator pass). Choosing `Accept and Commit` routes through a `Commit` agent node that stages and commits the theme artifact before ending the workflow.

The `Create` node uses `persist="full"` so the creator agent's LLM session is reused across revision loops, avoiding the cost of re-exploring the workspace and re-reading files on every iteration. The `Review` node intentionally does not persist its session — a fresh session on each pass gives the reviewer unbiased "fresh eyes" on the current draft, avoiding anchoring on prior assessments that could mask regressions. The artifact being reviewed is a single file, so the re-read cost is low. A graph-wide `max-session-turns` default of 10 caps context growth.

```dot
digraph theme_creation_iterative {
  node [max-session-turns="10"]

  Start -> Create

  Create [agent="theme-creator", prompt-ref="#creator-prompt", persist="full"]
  Create -> Review

  Review [agent="theme-reviewer", prompt-ref="#reviewer-prompt"]
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

Create or update a theme for the goal:

$goal

Before starting, use `workflow_get_output` to check for reviewer feedback from a previous iteration. If feedback is present, use it to revise the existing draft instead of starting over. If you disagree with a specific finding, you may provide a reasoned rebuttal instead of incorporating it.

Also use `workflow_get_context` with key "human.feedback" to check for human revision notes and incorporate those as well.

## `reviewer-prompt`

Review the current theme draft for the goal:

$goal

If the draft is acceptable, choose the Accept branch. If the draft needs changes, choose the Revise branch and provide specific revision feedback in your response.

## `human-review-interview`

```yaml #human-review-interview
preamble: |
  The theme-reviewer agent has approved the current theme draft.
  Please review the theme and decide whether to accept it or send it back for revision.

questions:
  - header: Decision
    question: Is the theme acceptable?
    type: single-select
    options:
      - label: Accept and Commit
      - label: Accept
      - label: Revise
    store: human.decision

  - header: Revision Notes
    question: What specific changes or improvements should be made?
    store: human.feedback
    show-if: "human.decision == Revise"
```

## `commit-prompt`

Commit the theme artifact.

Theme goal: $goal

**Step 1: stage changes**

Use the shell tool to review uncommitted changes with `git status` and `git diff --stat`.
Stage the theme files. These are typically a theme.css and associated files in a theme
directory. Use the goal description as a guide, but include any other files that are
clearly part of this theme creation work. Avoid staging unrelated changes.

**Step 2: commit**

Compose a commit message based on the theme goal and the actual changes staged.
Inspect the repository's recent commit history (`git log --oneline -20`) to infer the
project's commit message conventions and follow them. Also check for any commit message
instructions in the system prompt or prior context and apply those.
Run `git commit` with the composed message.

If any step fails (nothing to commit, git errors, etc.), report the issue but do not block
the workflow — execution will continue regardless.

---

This page was generated from [`.stencila/workflows/theme-creation-iterative/WORKFLOW.md`](https://github.com/stencila/stencila/blob/main/.stencila/workflows/theme-creation-iterative/WORKFLOW.md).
