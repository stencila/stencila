---
name: agent-creation-iterative
description: Create and iteratively refine a Stencila agent using the agent-creator and agent-reviewer agents, route approved drafts through human review, and loop on requested revisions
goal: Produce an acceptable Stencila agent definition for the requested purpose
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

```dot
digraph agent_creation_iterative {
  Start -> Create

  Create [agent="agent-creator", prompt-ref="#creator-prompt"]
  Create -> Review

  Review [agent="agent-reviewer", prompt-ref="#reviewer-prompt"]
  Review -> HumanReview  [label="Accept"]
  Review -> Create       [label="Revise"]

  HumanReview [interview-ref="#human-review-interview"]
  HumanReview -> End     [label="Accept"]
  HumanReview -> Create  [label="Revise"]
}
```

```text #creator-prompt
Create or update a Stencila agent that helps users accomplish this underlying task: $goal

Interpret that as the end-user objective the agent should support, not as an instruction to create another agent. Ignore workflow-process phrasing such as iteration, review loops, or acceptance criteria unless it is genuinely part of the domain task.

If reviewer feedback is present, use it to revise the existing draft instead of starting over:
$last_output

If human feedback is present, incorporate it as well:
$human.feedback
```

```text #reviewer-prompt
Review the current agent draft for the goal '$goal'. Ensure the agent addresses the underlying user task rather than accidentally becoming a meta-agent about creating agents.

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
      - label: Accept
      - label: Revise
    store: human.decision
    finish-if: Accept

  - header: Revision Notes
    question: What specific changes or improvements should be made?
    store: human.feedback
```

The workflow first uses the `agent-creator` agent to draft or revise the agent, then passes the draft to the `agent-reviewer` agent for review. The reviewer uses the `set_preferred_label` tool (provided automatically) to choose between the `Accept` and `Revise` edge labels; when it chooses Revise its response text contains concrete revision feedback that is routed back to `Create` via `$last_output`. The `Create` node consumes reviewer feedback from `$last_output` and any stored human revision notes from `$human.feedback`, so both automated and human guidance are available on iterative passes. After the reviewer accepts, the workflow enters a structured human review interview. Routing from `HumanReview` is driven by the first multiple-choice question's option labels, which intentionally match the outgoing edge labels `Accept` and `Revise`. The decision question uses `finish-if: Accept` to end the interview immediately when the agent is accepted, skipping the revision notes question. Choosing Revise continues the interview to collect feedback (stored as `human.feedback` for the next creator pass); on later iterations, `$human.feedback` is expected to contain the latest stored revision notes unless a subsequent run replaces them.
