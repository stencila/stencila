---
name: skill-creation-iterative
description: Create and iteratively refine a Stencila skill using the skill-creator and skill-reviewer agents, then continue through human review until accepted
goal: Produce an acceptable Stencila skill definition for the requested purpose
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

```dot
digraph skill_creation_iterative {
    Start -> Create -> Review
    Review -> HumanReview  [label="Accept", condition="context.last_output=yes"]
    Review -> Create       [label="Revise", condition="context.last_output!=yes"]
    HumanReview -> End     [label="Accept"]
    HumanReview -> Create  [label="Revise"]

    Create      [agent="skill-creator", prompt="Create or update a Stencila skill that helps users accomplish this underlying task: $goal\n\nInterpret that as the end-user objective the skill should support, not as an instruction to create another skill. Ignore workflow-process phrasing such as iteration, review loops, or acceptance criteria unless it is genuinely part of the domain task.\n\nIf the previous step provided review feedback, use it to revise the existing draft instead of starting over. Reviewer or human feedback to address:\n$last_output"]

    Review      [agent="skill-reviewer", prompt="Review the current skill draft for the goal '$goal'. Ensure the skill addresses the underlying user task rather than accidentally becoming a meta-skill about creating skills.\n\nIf the draft is acceptable, reply with ONLY yes in lowercase.\nIf the draft is not acceptable, reply with concrete revision feedback that the creator can use on the next pass."]

    HumanReview [ask="Is the skill acceptable after reviewer approval?"]
}
```

The workflow first uses the `skill-creator` agent to draft or revise the skill, then uses a review step that emits a deterministic routing signal via `context.last_output`: `yes` means the draft is acceptable and any other output is treated as revision feedback and routed back to `Create`. The `Create` node also consumes `$last_output`, so reviewer or human feedback from the previous step is available to the creator on iterative passes. After the reviewer approves, the workflow enters a binary human review gate: the human can either accept the skill to finish the workflow or send it back to `Create` for another agent-assisted revision cycle.
