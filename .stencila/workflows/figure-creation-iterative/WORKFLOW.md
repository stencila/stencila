---
name: figure-creation-iterative
title: Figure Creation Iterative Workflow
description: Create and iteratively refine a Stencila figure using the `figure-creator` and `figure-reviewer` agents, route approved drafts through human review with optional commit, and loop on requested revisions
goal-hint: Describe the figure you want to create or update — what document, figure type, content, or visual changes are needed?
keywords:
  - figure
  - creation
  - review
  - iterative
  - human-in-the-loop
  - stencila markdown
  - caption
  - subfigure
  - svg
when-to-use:
  - when a new Stencila figure needs to be drafted, inserted, or refined before acceptance
  - when you want agent-based figure authoring followed by explicit human approval cycles
  - when the task involves image figures, executable figures, captions, subfigures, layouts, or SVG overlays
when-not-to-use:
  - when you only need a one-pass figure change without review loops
  - when the task is to review an existing figure without modifying it
  - when the task is broader document writing rather than figure-focused creation or revision
---

This workflow first uses the `figure-creator` agent to draft or revise a Stencila figure, then passes the result to the `figure-reviewer` agent for review. The reviewer uses the `workflow_set_route` tool (provided automatically) to choose between the `Accept` and `Revise` edge labels; when it chooses Revise its response text contains concrete revision feedback.

The `Create` node uses the `workflow_get_output` tool to retrieve reviewer feedback and `workflow_get_context` with key `human.feedback` to retrieve human revision notes, so both automated and human guidance are available on iterative passes without bloating the prompt. After the reviewer accepts, the workflow enters a structured human review interview. Choosing Revise continues the interview to collect feedback (stored as `human.feedback` for the next creator pass) and loops back to `Create`. Choosing `Accept and Commit` routes through a `Commit` agent node that stages and commits the figure-related changes before ending the workflow.

The `Create` node uses `persist="full"` so the creator agent's LLM session is reused across revision loops, avoiding the cost of re-exploring the workspace and re-reading figure-related files on every iteration. The `Review` node intentionally does not persist its session — a fresh session on each pass gives the reviewer unbiased "fresh eyes" on the current draft, avoiding anchoring on prior assessments that could mask regressions. A graph-wide `max-session-turns` default of 10 caps context growth.

```dot
digraph figure_creation_iterative {
  node [max-session-turns="10"]

  Start -> Create

  Create [agent="figure-creator", prompt-ref="#creator-prompt", persist="full"]
  Create -> Review

  Review [agent="figure-reviewer", prompt-ref="#reviewer-prompt"]
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

```markdown #creator-prompt
Create or update a figure for the goal:

$goal

Before starting, use `workflow_get_output` to check for reviewer feedback from a previous iteration. If feedback is present, use it to revise the existing draft instead of starting over. If you disagree with a specific finding, you may provide a reasoned rebuttal instead of incorporating it.

Also use `workflow_get_context` with key "human.feedback" to check for human revision notes and incorporate those as well.
```

```markdown #reviewer-prompt
Review the current figure draft for the goal:

$goal

If the draft is acceptable, choose the Accept branch. If the draft needs changes, choose the Revise branch and provide specific revision feedback in your response.
```

```yaml #human-review-interview
preamble: |
  The figure-reviewer agent has approved the current figure draft.
  Please review the figure and decide whether to accept it or send it back for revision.

questions:
  - header: Decision
    question: Is the figure acceptable?
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

```markdown #commit-prompt
Commit the figure creation artifact.

Figure goal: $goal

**Step 1: stage changes**

Use the shell tool to review uncommitted changes with `git status` and `git diff --stat`.
Stage the figure-related files. These may include Stencila Markdown documents, image assets,
SVG overlays, or other files clearly part of this figure creation work. Use the goal description
as a guide, but avoid staging unrelated changes.

**Step 2: commit**

Compose a commit message based on the figure goal and the actual changes staged.
Inspect the repository's recent commit history (`git log --oneline -20`) to infer the
project's commit message conventions and follow them. Also check for any commit message
instructions in the system prompt or prior context and apply those.
Run `git commit` with the composed message.

If any step fails (nothing to commit, git errors, etc.), report the issue but do not block
the workflow — execution will continue regardless.
```
