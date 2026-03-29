---
name: workflow-creation-topdown
title: Workflow Creation Top-Down Workflow
description: Design a workflow top-down by first planning its structure and dependencies, creating each dependency via the appropriate child workflow, then building and refining the parent workflow with those dependencies available
goal-hint: Describe the workflow you want to design — what process should it automate, and what agents or child workflows will it need?
keywords:
  - workflow
  - creation
  - top-down
  - dependencies
  - design-first
  - iterative
  - human-in-the-loop
when-to-use:
  - when creating a workflow that needs custom agents or child workflows that do not yet exist
  - when you want to design the workflow structure first then create its dependencies before the workflow itself
  - when the dependency set should be reviewed and approved before investing in creation
when-not-to-use:
  - when the workflow's agents and child workflows already exist (use workflow-creation-iterative instead)
  - when you need a single agent or skill created without a workflow (use the agent/skill creation workflows directly)
  - when you need a throwaway workflow executed immediately (use workflow-create-run instead)
  - when you only need a one-pass workflow draft without review loops
---

This workflow supports top-down workflow creation — designing the workflow and its needed dependencies before building anything. It proceeds in four phases:

1. **Design**: The `workflow-creator` agent drafts a high-level workflow design: the pipeline structure, which agents it references, which child workflows it composes, and which of those are new vs. already available. For each new dependency it produces a name, description, and brief rationale. It stores three JSON arrays in the pipeline context for the downstream fan-outs:
   - `agents-with-skills` — agents that need custom skills (created via `agent-creation-topdown`)
   - `agents-without-skills` — agents that use existing skills or need none (created via `agent-creation-iterative`)
   - `child-workflows` — child workflows to be composed (created via `workflow-creation-iterative`)

2. **Design review**: A human reviews the proposed pipeline structure and dependency lists. Approving commits to creating all dependencies.

3. **Parallel dependency creation**: Three sequential fan-out/fan-in passes create all dependencies. Each pass handles one dependency type and routes to the appropriate child workflow:
   - Agents needing custom skills → `agent-creation-topdown`
   - Agents without custom skills → `agent-creation-iterative`
   - Child workflows → `workflow-creation-iterative`

   Each child workflow receives the dependency's name and description as its goal. Fan-in nodes collect results before proceeding to the next pass. Empty arrays produce no fan-out branches, so passes with no dependencies of that type are effectively skipped. Recursion is capped at one level — child workflows created here use `workflow-creation-iterative` (flat) and must reference existing or just-created agents.

4. **Workflow creation and review**: With all dependencies now in the workspace, the `workflow-creator` agent creates the actual WORKFLOW.md referencing those agents and child workflows. The `workflow-reviewer` reviews it, choosing Accept or Revise. After reviewer acceptance, a human review interview decides acceptance, acceptance with commit, or revision — the same tail pattern as `agent-creation-topdown` and `workflow-creation-iterative`.

The `Design` node uses `context-writable=true` so it can store the dependency lists via `workflow_set_context`, and also uses `workflow_get_output` and `workflow_get_context` to pick up feedback from previous iterations.

```dot
digraph workflow_creation_topdown {
  Start -> Design

  Design [agent="workflow-creator", prompt-ref="#design-prompt", context-writable=true]
  Design -> DesignReview

  DesignReview [interview-ref="#design-review-interview"]
  DesignReview -> FanOutAgentsWithSkills [label="Approve"]
  DesignReview -> Design                 [label="Revise"]

  FanOutAgentsWithSkills [fan-out="agents-with-skills", label="Create agents needing custom skills"]
  FanOutAgentsWithSkills -> CreateAgentWithSkills

  CreateAgentWithSkills [workflow="agent-creation-topdown", prompt-ref="#create-agent-with-skills-prompt", goal="Create agent '$fan_out.item.name': $fan_out.item.description"]
  CreateAgentWithSkills -> FanInAgentsWithSkills

  FanInAgentsWithSkills [label="Collect agents with skills"]
  FanInAgentsWithSkills -> FanOutAgentsWithoutSkills

  FanOutAgentsWithoutSkills [fan-out="agents-without-skills", label="Create agents without custom skills"]
  FanOutAgentsWithoutSkills -> CreateAgentWithoutSkills

  CreateAgentWithoutSkills [workflow="agent-creation-iterative", prompt-ref="#create-agent-without-skills-prompt", goal="Create agent '$fan_out.item.name': $fan_out.item.description"]
  CreateAgentWithoutSkills -> FanInAgentsWithoutSkills

  FanInAgentsWithoutSkills [label="Collect agents without skills"]
  FanInAgentsWithoutSkills -> FanOutChildWorkflows

  FanOutChildWorkflows [fan-out="child-workflows", label="Create child workflows"]
  FanOutChildWorkflows -> CreateChildWorkflow

  CreateChildWorkflow [workflow="workflow-creation-iterative", prompt-ref="#create-child-workflow-prompt", goal="Create workflow '$fan_out.item.name': $fan_out.item.description"]
  CreateChildWorkflow -> FanInChildWorkflows

  FanInChildWorkflows [label="Collect child workflows"]
  FanInChildWorkflows -> CreateWorkflow

  CreateWorkflow [agent="workflow-creator", prompt-ref="#create-workflow-prompt"]
  CreateWorkflow -> ReviewWorkflow

  ReviewWorkflow [agent="workflow-reviewer", prompt-ref="#review-workflow-prompt"]
  ReviewWorkflow -> HumanReview     [label="Accept"]
  ReviewWorkflow -> CreateWorkflow  [label="Revise"]

  HumanReview [interview-ref="#human-review-interview"]
  HumanReview -> Commit             [label="Accept and Commit"]
  HumanReview -> End                [label="Accept"]
  HumanReview -> CreateWorkflow     [label="Revise"]

  Commit [agent="general", prompt-ref="#commit-prompt"]
  Commit -> End
}
```

```markdown #design-prompt
Design a workflow for the following goal. Do NOT create the workflow yet — produce a design document that covers:

1. The workflow's name, purpose, and description
2. The pipeline structure: what stages/nodes the workflow will have, how they connect, and what each stage does
3. A list of agents the workflow will reference, with each agent's name and a one-line description. For each agent, note whether it needs custom skills that do not yet exist (needs-skills: true) or can work with existing skills or none (needs-skills: false)
4. A list of child workflows the workflow will compose (if any), with each child workflow's name and a one-line description
5. Which of the above agents and child workflows already exist in the workspace vs. which need to be created
6. The workflow's intended goal, goal-hint, keywords, and when-to-use/when-not-to-use signals

Goal:

$goal

Before starting, use `workflow_get_output` to check for feedback from a previous iteration. If feedback is present, revise the design accordingly rather than starting over.

Also use `workflow_get_context` with key "human.feedback" to check for human revision notes and incorporate those as well.

After producing the design document, you MUST call `workflow_set_context` three times to store the dependency lists. Use the following keys and provide JSON arrays of objects:

1. Key "agents-with-skills" — agents that need custom skills. Each object: `{"name": "agent-name", "description": "one-line description", "skills-needed": "brief description of skills"}`
2. Key "agents-without-skills" — agents that use existing skills or need none. Each object: `{"name": "agent-name", "description": "one-line description"}`
3. Key "child-workflows" — child workflows to compose. Each object: `{"name": "workflow-name", "description": "one-line description"}`

Use empty arrays `[]` for any category with no new dependencies. Only include dependencies that need to be CREATED — omit agents and workflows that already exist in the workspace.

These arrays are used by the downstream fan-outs to create all dependencies in parallel before the workflow itself is built.
```

```yaml #design-review-interview
preamble: |
  The workflow design is ready for review. It includes the pipeline structure,
  a list of agents (with and without custom skills), and any child workflows
  to be composed.

  Please review the design and dependency lists carefully — approving will
  begin creating all dependencies:
  - Agents needing custom skills via `agent-creation-topdown`
  - Agents without custom skills via `agent-creation-iterative`
  - Child workflows via `workflow-creation-iterative`

questions:
  - header: Decision
    question: Is the workflow design and dependency list ready to proceed with dependency creation?
    type: single-select
    options:
      - label: Approve
      - label: Revise
    store: human.decision
    finish-if: Approve

  - header: Revision Notes
    question: What changes should be made to the workflow design or dependency list?
    store: human.feedback
    show-if: "human.decision == Revise"
```

```markdown #create-agent-with-skills-prompt
Create the following agent (it needs custom skills that will be designed and built as part of this process):

Name: $fan_out.item.name

Description: $fan_out.item.description

Skills needed: $fan_out.item.skills-needed

Create an agent with the specified name, description, and custom skills. The agent should be complete, well-structured, and ready for the workflow to reference.
```

```markdown #create-agent-without-skills-prompt
Create the following agent:

Name: $fan_out.item.name

Description: $fan_out.item.description

Create an agent with the specified name and description. The agent should be complete, well-structured, and ready for the workflow to reference.
```

```markdown #create-child-workflow-prompt
Create the following workflow:

Name: $fan_out.item.name

Description: $fan_out.item.description

Create a workflow with the specified name and description. The workflow should be complete, well-structured, and ready to be composed as a child workflow.
```

```markdown #create-workflow-prompt
Create the workflow defined in the approved design, now that all its dependencies have been created.

Use `workflow_get_output` to check for reviewer feedback from a previous iteration. If feedback is present, use it to revise the existing workflow draft instead of starting over. If you disagree with a specific finding, you may provide a reasoned rebuttal instead of incorporating it.

Also use `workflow_get_context` with key "human.feedback" to check for human revision notes and incorporate those as well.

The workflow's goal is:

$goal

Reference the agents and child workflows that were created in the earlier phases of this workflow. Make sure all dependencies from the approved design are correctly referenced in the pipeline.
```

```markdown #review-workflow-prompt
Review the current workflow draft for the goal:

$goal

The workflow was designed top-down: its agent and child workflow dependencies were created first based on an approved design, then the workflow was created to reference those dependencies.

Verify that:

1. The workflow correctly references all agents and child workflows from the approved design
2. The pipeline structure matches the approved design
3. The workflow's instructions, prompts, and node configuration are clear and complete
4. The frontmatter (description, keywords, when-to-use, when-not-to-use) is accurate and helpful
5. The DOT graph is syntactically valid and follows Stencila workflow conventions

If the draft is acceptable, choose the Accept branch. If the draft needs changes, choose the Revise branch and provide specific revision feedback in your response.
```

```yaml #human-review-interview
preamble: |
  The `workflow-reviewer` has approved the workflow draft.
  The workflow was created top-down: agents and child workflows were designed
  and built first, then the workflow was created to reference them.

  Please review the final workflow and decide whether to accept it or
  send it back for revision.

questions:
  - header: Decision
    question: Is the workflow acceptable?
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
Commit the workflow and dependency artifacts created by this workflow.

Workflow goal: $goal

**Step 1: stage changes**

Use the shell tool to review uncommitted changes with `git status` and `git diff --stat`.
Stage the workflow files and any agent, skill, or child workflow files created during this
pipeline. These are typically directories under `.stencila/workflows/`, `.stencila/agents/`,
and `.stencila/skills/`. Use the goal description as a guide, but include any other files
that are clearly part of this workflow creation work. Avoid staging unrelated changes.

**Step 2: commit**

Compose a commit message based on the workflow goal and the actual changes staged.
Inspect the repository's recent commit history (`git log --oneline -20`) to infer the
project's commit message conventions and follow them. Also check for any commit message
instructions in the system prompt or prior context and apply those.
Run `git commit` with the composed message.

If any step fails (nothing to commit, git errors, etc.), report the issue but do not block
the workflow — execution will continue regardless.
```
