---
title: "Agent Creation Top-Down Workflow"
description: "Design an agent top-down by first planning its skills, creating each skill via the `skill-creation-iterative` workflow, and then creating, refining, and optionally committing the agent with those skills available"
keywords:
  - agent
  - creation
  - top-down
  - skills
  - design-first
  - iterative
  - human-in-the-loop
---

Design an agent top-down by first planning its skills, creating each skill via the `skill-creation-iterative` workflow, and then creating, refining, and optionally committing the agent with those skills available

**Keywords:** agent · creation · top-down · skills · design-first · iterative · human-in-the-loop

> [!tip] Usage
>
> To run this workflow, start your prompt with `~agent-creation-topdown` followed by your goal, or select it with the `/workflow` command.

# When to use

- when creating a new agent that needs custom skills which do not yet exist
- when you want to design the agent concept first then create its skills before the agent itself
- when the agent's skill requirements should be reviewed and approved before investing in skill creation

# When not to use

- when the agent's skills already exist and only the agent definition is needed (use agent-creation-iterative instead)
- when you need a single skill created without an agent (use skill-creation-iterative instead)
- when you only need a one-pass agent draft without review loops

# Configuration

| Property | Value |
| -------- | ----- |
| Goal | Describe the agent you want to design — what should it do, and what specialized skills will it need? |
| Referenced agents | [`agent-creator`](/docs/agents/builtin/agents/agent-creator/), [`agent-reviewer`](/docs/agents/builtin/agents/agent-reviewer/), [`general`](/docs/agents/builtin/general/general/) |

# Pipeline

This workflow supports top-down agent creation — designing the agent and its needed skills before building anything. It proceeds in four phases:

1. **Design**: The `agent-creator` agent drafts a high-level agent design that includes the agent's purpose, the skills it needs, and a brief description of each skill. This is a design document, not the final AGENT.md. The design phase explicitly considers whether a single-skill or multi-skill agent is appropriate — Stencila optimizes single-skill agents by preloading the skill into the agent's prompt, so single-skill agents are preferred unless skill scope or reuse considerations justify multiple skills. The agent also stores the skills list as a JSON array in the pipeline context (key `skills`) for the downstream fan-out.
2. **Design review**: A human reviews and approves (or revises) the agent design and skill list before any skills are created.
3. **Parallel skill creation**: After approval, the workflow fans out over the `skills` list stored in the pipeline context, spawning one `skill-creation-iterative` child workflow per skill in parallel. Each child workflow handles its own create-review-human-approve cycle independently. A fan-in node collects the results before proceeding.
4. **Agent creation and review**: With all skills now available in the workspace, the `agent-creator` agent creates the actual AGENT.md referencing those skills. The `agent-reviewer` then reviews it, using the `workflow_set_route` tool to choose between the `Accept` and `Revise` edge labels; when it chooses Revise its response text contains concrete revision feedback. After the reviewer accepts, a final human review interview decides acceptance, acceptance with commit, or revision. Choosing "Accept and Commit" routes through a Commit agent node that stages and commits the agent and skill artifacts before ending the workflow.

The `Design` node uses `context-writable=true` so it can store the skills list via `workflow_set_context`, and also uses `workflow_get_output` and `workflow_get_context` to pick up feedback from previous iterations, matching the pattern used by the sibling `agent-creation-iterative` and `skill-creation-iterative` workflows.

The `Design` and `CreateAgent` nodes use `persist="full"` so the creator agent's LLM session is reused across revision loops, avoiding the cost of re-exploring the workspace, re-reading files, and re-discovering conventions on every iteration. The `ReviewAgent` node intentionally does not persist its session — a fresh session on each pass gives the reviewer unbiased "fresh eyes" on the current draft, avoiding anchoring on prior assessments that could mask regressions. The artifact being reviewed is a single file, so the re-read cost is low. A graph-wide `max-session-turns` default of 10 caps context growth.

```dot
digraph agent_creation_topdown {
  node [max-session-turns="10"]

  Start -> Design

  Design [agent="agent-creator", prompt-ref="#design-prompt", context-writable=true, persist="full"]
  Design -> DesignReview

  DesignReview [interview-ref="#design-review-interview"]
  DesignReview -> FanOutSkills [label="Approve"]
  DesignReview -> Design       [label="Revise"]

  FanOutSkills [fan-out="skills", label="Create each skill in parallel"]
  FanOutSkills -> CreateSkill

  CreateSkill [workflow="skill-creation-iterative", prompt-ref="#create-skill-prompt"]
  CreateSkill -> FanInSkills

  FanInSkills [label="Collect created skills"]
  FanInSkills -> CreateAgent

  CreateAgent [agent="agent-creator", prompt-ref="#create-agent-prompt", persist="full"]
  CreateAgent -> ReviewAgent

  ReviewAgent [agent="agent-reviewer", prompt-ref="#review-agent-prompt"]
  ReviewAgent -> HumanReview  [label="Accept"]
  ReviewAgent -> CreateAgent  [label="Revise"]

  HumanReview [interview-ref="#human-review-interview"]
  HumanReview -> Commit       [label="Accept and Commit"]
  HumanReview -> End          [label="Accept"]
  HumanReview -> CreateAgent  [label="Revise"]

  Commit [agent="general", prompt-ref="#commit-prompt"]
  Commit -> End
}
```

## `design-prompt`

Design an agent for the following goal. Do NOT create the agent yet — produce a design document that covers:

1. The agent's name, purpose, and description
2. A list of skills the agent will need, with each skill's name, a one-line description, and the key capabilities it should provide
3. Any existing workspace skills that can be reused (list them if known)
4. The agent's intended tools, model preferences, and any other configuration
5. A rationale for the number of skills — prefer a single-skill agent when possible, because Stencila optimizes single-skill agents by preloading the skill directly into the agent's prompt. Only design a multi-skill agent when the skills have clearly distinct scopes or are intended for reuse across different agents. If you propose multiple skills, explain why they should not be combined into one.

Goal:

$goal

Before starting, use `workflow_get_output` to check for feedback from a previous iteration. If feedback is present, revise the design accordingly rather than starting over.

Also use `workflow_get_context` with key "human.feedback" to check for human revision notes and incorporate those as well.

After producing the design document, you MUST call `workflow_set_context` to store the skills list. Use key "skills" and provide a JSON array of objects, where each object has "name" (kebab-case skill name), "description" (one-line description), and "capabilities" (key capabilities the skill should provide). For example:

`[{"name": "data-analysis", "description": "Analyze datasets and produce summaries", "capabilities": "Statistical analysis, visualization, trend detection"}]`

This array is used by the downstream fan-out to create all skills in parallel.

## `design-review-interview`

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
    show-if: "human.decision == Revise"
```

## `create-skill-prompt`

Create the following skill:

Name: $fan_out.item.name

Description: $fan_out.item.description

Capabilities: $fan_out.item.capabilities

Create a skill with the specified name, description, and capabilities. The skill should be complete, well-structured, and ready for the agent to reference.

## `create-agent-prompt`

Create the agent defined in the approved design, now that all its skills have been created.

Use `workflow_get_output` to check for reviewer feedback from a previous iteration. If feedback is present, use it to revise the existing agent draft instead of starting over. If you disagree with a specific finding, you may provide a reasoned rebuttal instead of incorporating it.

Also use `workflow_get_context` with key "human.feedback" to check for human revision notes and incorporate those as well.

The agent's goal is:

$goal

Reference the skills that were created in the earlier phase of this workflow. Make sure each skill is listed in the agent's skills section.

## `review-agent-prompt`

Review the current agent draft for the goal:

$goal

The agent was designed top-down: its skills were created first based on an approved design, then the agent was created to reference those skills.

Verify that:

1. The agent correctly references all skills from the approved design
2. The agent's instructions are clear, complete, and consistent with its skills
3. The agent's tools, model preferences, and other configuration are appropriate
4. The skill count is justified — single-skill agents are preferred because Stencila preloads the skill into the prompt; flag any multi-skill design that could reasonably be consolidated

If the draft is acceptable, choose the Accept branch. If the draft needs changes, choose the Revise branch and provide specific revision feedback in your response.

## `human-review-interview`

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

Commit the agent and skill artifacts created by this workflow.

Agent goal: $goal

**Step 1: stage changes**

Use the shell tool to review uncommitted changes with `git status` and `git diff --stat`.
Stage the agent and skill files. These are typically directories under `.stencila/agents/`
and `.stencila/skills/`. Use the goal description as a guide, but include any other files
that are clearly part of this agent creation work. Avoid staging unrelated changes.

**Step 2: commit**

Compose a commit message based on the agent goal and the actual changes staged.
Inspect the repository's recent commit history (`git log --oneline -20`) to infer the
project's commit message conventions and follow them. Also check for any commit message
instructions in the system prompt or prior context and apply those.
Run `git commit` with the composed message.

If any step fails (nothing to commit, git errors, etc.), report the issue but do not block
the workflow — execution will continue regardless.

---

This page was generated from [`.stencila/workflows/agent-creation-topdown/WORKFLOW.md`](https://github.com/stencila/stencila/blob/main/.stencila/workflows/agent-creation-topdown/WORKFLOW.md).
