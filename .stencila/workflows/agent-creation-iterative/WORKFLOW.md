---
name: agent-creation-iterative
title: Agent Creation Iterative Workflow
description: Create and iteratively refine a Stencila agent using the `agent-creator` and `agent-reviewer` agents, route approved drafts through human review with optional commit, and loop on requested revisions
goal-hint: Describe the agent you want to create — what should it help users do?
keywords:
  - agent
  - creation
  - review
  - iterative
  - human-in-the-loop
when-to-use:
  - when a new Stencila agent needs to be drafted, reviewed, and refined before acceptance
  - when you want agent-based agent authoring followed by explicit human approval cycles
when-not-to-use:
  - when you only need a one-pass agent draft without review loops
  - when the task is to review an existing agent without creating or refining it
---

This workflow first uses the `agent-creator` agent to draft or revise the agent, then passes the draft to the `agent-reviewer` agent for review. The reviewer uses the `workflow_set_route` tool to choose between the `Accept` and `Revise` edge labels; when it chooses Revise its response text contains concrete revision feedback. The `Create` node uses the `workflow_get_output` tool to retrieve reviewer feedback and `workflow_get_context` with key `human.feedback` to retrieve human revision notes, so both automated and human guidance are available on iterative passes without bloating the prompt. After the reviewer accepts, the workflow enters a structured human review interview. Routing from `HumanReview` is driven by the first multiple-choice question's option labels, which intentionally match the outgoing edge labels `Accept` and `Revise`. Choosing Revise continues the interview to collect feedback (stored as `human.feedback` for the next creator pass). Choosing "Accept and Commit" routes through a Commit agent node that stages and commits the agent artifact before ending the workflow.

```dot
digraph agent_creation_iterative {
  Start -> Create

  Create [agent="agent-creator", prompt-ref="#creator-prompt"]
  Create -> Review

  Review [agent="agent-reviewer", prompt-ref="#reviewer-prompt"]
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
Create or update an agent for the goal:

$goal

Before starting, use `workflow_get_output` to check for reviewer feedback from a previous iteration. If feedback is present, use it to revise the existing draft instead of starting over. If you disagree with a specific finding, you may provide a reasoned rebuttal instead of incorporating it.

Also use `workflow_get_context` with key "human.feedback" to check for human revision notes and incorporate those as well.
```

```markdown #reviewer-prompt
Review the current agent draft for the goal:

$goal

If the draft is acceptable, choose the Accept branch. If the draft needs changes, choose the Revise branch and provide specific revision feedback in your response.
```

```yaml #human-review-interview
preamble: |
  The agent-reviewer agent has approved the current draft.
  Please review the agent and decide whether to accept it or send it back for revision.

questions:
  - header: Decision
    question: Is the agent acceptable?
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
Commit the agent artifact.

Agent goal: $goal

**Step 1: stage changes**

Use the shell tool to review uncommitted changes with `git status` and `git diff --stat`.
Stage the agent files. These are typically an AGENT.md and associated files in a directory
under `.stencila/agents/`. Use the goal description as a guide, but include any other files
that are clearly part of this agent creation work. Avoid staging unrelated changes.

**Step 2: commit**

Compose a commit message based on the agent goal and the actual changes staged.
Inspect the repository's recent commit history (`git log --oneline -20`) to infer the
project's commit message conventions and follow them. Also check for any commit message
instructions in the system prompt or prior context and apply those.
Run `git commit` with the composed message.

If any step fails (nothing to commit, git errors, etc.), report the issue but do not block
the workflow — execution will continue regardless.
```
