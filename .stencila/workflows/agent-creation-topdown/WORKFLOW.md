---
name: agent-creation-topdown
description: Design an agent top-down by first planning its skills, creating each skill via the skill-creation-iterative workflow, and then creating and refining the agent with those skills available
goal-hint: Describe the agent you want to design — what should it do, and what specialized skills will it need?
keywords:
  - agent
  - creation
  - top-down
  - skills
  - design-first
  - iterative
  - human-in-the-loop
when-to-use:
  - when creating a new agent that needs custom skills which do not yet exist
  - when you want to design the agent concept first then create its skills before the agent itself
  - when the agent's skill requirements should be reviewed and approved before investing in skill creation
when-not-to-use:
  - when the agent's skills already exist and only the agent definition is needed (use agent-creation-iterative instead)
  - when you need a single skill created without an agent (use skill-creation-iterative instead)
  - when you only need a one-pass agent draft without review loops
---

This workflow supports top-down agent creation — designing the agent and its needed skills before building anything. It proceeds in four phases:

1. **Design**: The `agent-creator` agent drafts a high-level agent design that includes the agent's purpose, the skills it needs, and a brief description of each skill. This is a design document, not the final AGENT.md. The design phase explicitly considers whether a single-skill or multi-skill agent is appropriate — Stencila optimizes single-skill agents by preloading the skill into the agent's prompt, so single-skill agents are preferred unless skill scope or reuse considerations justify multiple skills. The agent also stores the skills list as a JSON array in the pipeline context (key `skills`) for the downstream fan-out.
2. **Design review**: A human reviews and approves (or revises) the agent design and skill list before any skills are created.
3. **Parallel skill creation**: After approval, the workflow fans out over the `skills` list stored in the pipeline context, spawning one `skill-creation-iterative` child workflow per skill in parallel. Each child workflow handles its own create-review-human-approve cycle independently. A fan-in node collects the results before proceeding.
4. **Agent creation and review**: With all skills now available in the workspace, the `agent-creator` agent creates the actual AGENT.md referencing those skills. The `agent-reviewer` then reviews it, using the `workflow_set_route` tool to choose between the `Accept` and `Revise` edge labels; when it chooses Revise its response text contains concrete revision feedback. After the reviewer accepts, a final human review interview decides acceptance or revision.

The `Design` node uses `context-writable=true` so it can store the skills list via `workflow_set_context`, and also uses `workflow_get_output` and `workflow_get_context` to pick up feedback from previous iterations, matching the pattern used by the sibling `agent-creation-iterative` and `skill-creation-iterative` workflows.

```dot
digraph agent_creation_topdown {
  Start -> Design

  Design [agent="agent-creator", prompt-ref="#design-prompt", context-writable=true]
  Design -> DesignReview

  DesignReview [interview-ref="#design-review-interview"]
  DesignReview -> FanOutSkills [label="Approve"]
  DesignReview -> Design       [label="Revise"]

  FanOutSkills [fan-out="skills", label="Create each skill in parallel"]
  FanOutSkills -> CreateSkill

  CreateSkill [workflow="skill-creation-iterative", prompt-ref="#create-skill-prompt"]
  CreateSkill -> FanInSkills

  FanInSkills [shape=tripleoctagon, label="Collect created skills"]
  FanInSkills -> CreateAgent

  CreateAgent [agent="agent-creator", prompt-ref="#create-agent-prompt"]
  CreateAgent -> ReviewAgent

  ReviewAgent [agent="agent-reviewer", prompt-ref="#review-agent-prompt"]
  ReviewAgent -> HumanReview  [label="Accept"]
  ReviewAgent -> CreateAgent  [label="Revise"]

  HumanReview [interview-ref="#human-review-interview"]
  HumanReview -> End          [label="Accept"]
  HumanReview -> CreateAgent  [label="Revise"]
}
```

```text #design-prompt
Design an agent for the following goal. Do NOT create the agent yet — produce a design document that covers:

1. The agent's name, purpose, and description
2. A list of skills the agent will need, with each skill's name, a one-line description, and the key capabilities it should provide
3. Any existing workspace skills that can be reused (list them if known)
4. The agent's intended tools, model preferences, and any other configuration
5. A rationale for the number of skills — prefer a single-skill agent when possible, because Stencila optimizes single-skill agents by preloading the skill directly into the agent's prompt. Only design a multi-skill agent when the skills have clearly distinct scopes or are intended for reuse across different agents. If you propose multiple skills, explain why they should not be combined into one.

Goal:

$goal

Before starting, use workflow_get_output to check for feedback from a previous iteration. If feedback is present, revise the design accordingly rather than starting over.

Also use workflow_get_context with key "human.feedback" to check for human revision notes and incorporate those as well.

After producing the design document, you MUST call workflow_set_context to store the skills list. Use key "skills" and provide a JSON array of objects, where each object has "name" (kebab-case skill name), "description" (one-line description), and "capabilities" (key capabilities the skill should provide). For example:

[{"name": "data-analysis", "description": "Analyze datasets and produce summaries", "capabilities": "Statistical analysis, visualization, trend detection"}]

This array is used by the downstream fan-out to create all skills in parallel.
```

```yaml #design-review-interview
preamble: |
  The agent design is ready for review. It includes the agent's purpose,
  a list of skills to be created, and configuration details.

  Note: Stencila optimizes single-skill agents by preloading the skill
  into the agent's prompt. Prefer single-skill agents unless skill scope
  or reuse across agents justifies multiple skills.

  Please review the design and skill list carefully — approving will
  begin creating all skills in parallel, each via its own run of the
  `skill-creation-iterative` workflow.

questions:
  - header: Decision
    question: Is the agent design and skill list ready to proceed with skill creation?
    type: single-select
    options:
      - label: Approve
      - label: Revise
    store: human.decision
    finish-if: Approve

  - header: Revision Notes
    question: What changes should be made to the agent design or skill list?
    store: human.feedback
```

```text #create-skill-prompt
Create the following skill:

Name: $fan_out.item.name
Description: $fan_out.item.description
Capabilities: $fan_out.item.capabilities

Create a skill with the specified name, description, and capabilities. The skill should be complete, well-structured, and ready for the agent to reference.
```

```text #create-agent-prompt
Create the agent defined in the approved design, now that all its skills have been created.

Use workflow_get_output to check for reviewer feedback from a previous iteration. If feedback is present, use it to revise the existing agent draft instead of starting over. If you disagree with a specific finding, you may provide a reasoned rebuttal instead of incorporating it.

Also use workflow_get_context with key "human.feedback" to check for human revision notes and incorporate those as well.

The agent's goal is:

$goal

Reference the skills that were created in the earlier phase of this workflow. Make sure each skill is listed in the agent's skills section.
```

```text #review-agent-prompt
Review the current agent draft for the goal:

$goal

The agent was designed top-down: its skills were created first based on an approved design, then the agent was created to reference those skills.

Verify that:
1. The agent correctly references all skills from the approved design
2. The agent's instructions are clear, complete, and consistent with its skills
3. The agent's tools, model preferences, and other configuration are appropriate
4. The skill count is justified — single-skill agents are preferred because Stencila preloads the skill into the prompt; flag any multi-skill design that could reasonably be consolidated

If the draft is acceptable, choose the Accept branch. If the draft needs changes, choose the Revise branch and provide specific revision feedback in your response.
```

```yaml #human-review-interview
preamble: |
  The `agent-reviewer` has approved the agent draft.
  The agent was created top-down: skills were designed and built first,
  then the agent was created to reference them.

  Please review the final agent and decide whether to accept it or
  send it back for revision.

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
