---
name: skill-creation-iterative
description: Create and iteratively refine a Stencila skill using the skill-creator and skill-reviewer agents, with human review, optional commit, and revision loops until accepted
goal-hint: Describe the skill you want to create — what capability should it give agents?
keywords:
  - skill
  - creation
  - review
  - iterative
  - human-in-the-loop
when-to-use:
  - when a new Stencila skill needs to be drafted, reviewed, and refined before acceptance
  - when you want agent-based skill authoring followed by explicit human approval cycles
when-not-to-use:
  - when you only need a one-pass skill draft without review loops
  - when the task is to review an existing skill without creating or refining it
---

This workflow first uses the `skill-creator` agent to draft or revise the skill, then passes the draft to the `skill-reviewer` agent for review. The reviewer uses the `workflow_set_route` tool to choose between the `Accept` and `Revise` edge labels; when it chooses Revise its response text contains concrete revision feedback. The `Create` node uses the `workflow_get_output` tool to retrieve reviewer feedback and `workflow_get_context` with key `human.feedback` to retrieve human revision notes, so both automated and human guidance are available on iterative passes without bloating the prompt. After the reviewer accepts, the workflow enters a structured human review interview. Choosing Revise continues the interview to collect feedback (stored as `human.feedback` for the next creator pass) and loops back to `Create`. Choosing "Accept and Commit" routes through a Commit agent node that stages and commits the skill artifact before ending the workflow.

```dot
digraph skill_creation_iterative {
  Start -> Create

  Create [agent="skill-creator", prompt-ref="#creator-prompt"]
  Create -> Review

  Review [agent="skill-reviewer", prompt-ref="#reviewer-prompt"]
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

```text #creator-prompt
Create or update a skill for the goal:

$goal

Before starting, use workflow_get_output to check for reviewer feedback from a previous iteration. If feedback is present, use it to revise the existing draft instead of starting over. If you disagree with a specific finding, you may provide a reasoned rebuttal instead of incorporating it.

Also use workflow_get_context with key "human.feedback" to check for human revision notes and incorporate those as well.
```

```text #reviewer-prompt
Review the current skill draft for the goal:

$goal

If the draft is acceptable, choose the Accept branch. If the draft needs changes, choose the Revise branch and provide specific revision feedback in your response.
```

```yaml #human-review-interview
preamble: |
  The skill-reviewer agent has approved the current draft.
  Please review the skill and decide whether to accept it or send it back for revision.

questions:
  - header: Decision
    question: Is the skill acceptable?
    type: single-select
    options:
      - label: Accept and Commit
      - label: Accept
      - label: Revise
    store: human.decision

  - header: Revision Notes
    question: What specific changes or improvements should be made?
    store: human.feedback
```

```text #commit-prompt
Commit the skill artifact.

Skill goal: $goal

Step 1 — stage changes:
  Use the shell tool to review uncommitted changes with `git status` and `git diff --stat`.
  Stage the skill files. These are typically a SKILL.md and associated files in a directory
  under `.stencila/skills/`. Use the goal description as a guide, but include any other files
  that are clearly part of this skill creation work. Avoid staging unrelated changes.

Step 2 — commit:
  Compose a commit message based on the skill goal and the actual changes staged.
  Inspect the repository's recent commit history (`git log --oneline -20`) to infer the
  project's commit message conventions and follow them. Also check for any commit message
  instructions in the system prompt or prior context and apply those.
  Run `git commit` with the composed message.

If any step fails (nothing to commit, git errors, etc.), report the issue but do not block
the workflow — execution will continue regardless.
```
