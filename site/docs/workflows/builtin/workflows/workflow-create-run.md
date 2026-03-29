---
title: "Workflow Create and Run Workflow"
description: "Generate an ephemeral workflow tailored to a user's goal and immediately execute it, enabling single-delegation dynamic workflow orchestration"
keywords:
  - workflow
  - dynamic
  - generation
  - execution
  - orchestration
  - ephemeral
  - single-delegation
---

Generate an ephemeral workflow tailored to a user's goal and immediately execute it, enabling single-delegation dynamic workflow orchestration

**Keywords:** workflow · dynamic · generation · execution · orchestration · ephemeral · single-delegation

> [!tip] Usage
>
> To run this workflow, start your prompt with `~workflow-create-run` followed by your goal, or select it with the `/workflow` command.

# When to use

- when a task needs a multi-step workflow but no existing workflow matches
- when the manager agent needs to create and run a workflow in a single delegation
- when the user describes a goal that would benefit from a custom pipeline but wants results, not a workflow artifact

# When not to use

- when an existing workflow already handles the task (use it directly)
- when the user wants to keep the workflow as a reusable artifact (use workflow-creation-iterative instead)
- when the task is simple enough for a single agent without orchestration

# Configuration

| Property | Value |
| -------- | ----- |
| Goal | What task or goal should be accomplished? Describe what you need done — a workflow will be created and executed to achieve it |
| Referenced agents | [`workflow-creator`](/docs/agents/builtin/workflows/workflow-creator/) |

# Pipeline

This workflow solves the problem of needing to both create and execute a workflow in a single delegation. It uses the `workflow-creator` agent to generate an ephemeral child workflow under the fixed name `workflow-create-run-temporary`, validates it, and then executes it via `workflow=` composition.

The `Create` node instructs the `workflow-creator` agent to design a workflow appropriate to the goal — including agent review stages, human gates, iterative loops, and branching where the task warrants them. The whole point of generating a workflow (rather than just running a single agent) is to get multi-stage orchestration with quality checks. The agent stores the child workflow's goal into pipeline context via `workflow_set_context` so the `Execute` node can pass it through. A shell node validates the generated workflow, and on failure the validation error is stored and fed back to the creator for a retry. On success, the child workflow is executed, and finally cleaned up.

The creator agent has `context-writable=true` so it can store the child goal via `workflow_set_context`. The `Execute` node passes the creator-decided goal to the child workflow. The `Cleanup` shell node removes the ephemeral child directory after execution regardless of outcome.

```dot
digraph workflow_create_run {
  Start -> Create

  Create [
    agent="workflow-creator",
    prompt-ref="#create-prompt",
    context-writable=true
  ]
  Create -> Validate

  Validate [
    shell="stencila workflows validate workflow-create-run-temporary 2>&1",
    store="validation_result"
  ]
  Validate -> Execute      [label="Pass", condition="outcome=success"]
  Validate -> Create       [label="Fail", condition="outcome!=success"]

  Execute [
    workflow="workflow-create-run-temporary",
    label="Execute generated workflow",
    goal="$child_goal"
  ]
  Execute -> Cleanup

  Cleanup [shell="rm -rf .stencila/workflows/workflow-create-run-temporary"]
  Cleanup -> End
}
```

```text #create-prompt
Generate an ephemeral workflow tailored to this goal:

$goal

CRITICAL INSTRUCTIONS — read all of these before starting:

1. The workflow MUST be created with the exact name "workflow-create-run-temporary" as an ephemeral
   workflow in the workspace. This means:
   - Directory: .stencila/workflows/workflow-create-run-temporary/
   - File: .stencila/workflows/workflow-create-run-temporary/WORKFLOW.md
   - A .gitignore file containing exactly "*" in .stencila/workflows/workflow-create-run-temporary/

2. Design the workflow to match the complexity of the goal:
   - The whole point of a workflow is multi-stage orchestration — include agent review stages,
     human approval gates, iterative revision loops, and branching where the task warrants them
   - Do not optimize for reusability (this is a one-shot ephemeral workflow), but do design
     for quality — review and iteration stages are what make workflows valuable over single agents
   - For substantial tasks (coding, writing, design), include create-review-revise patterns
   - For simpler tasks, a shorter pipeline is fine — but it should still have at least two
     meaningful stages, otherwise a single agent would suffice and no workflow is needed
   - Use human gates when the goal involves consequential actions (deployment, publishing,
     committing) or when the user would reasonably want to approve intermediate results

3. Set the child workflow's goal in the frontmatter to a clear, actionable description of
   what the workflow should accomplish. Then store this same goal string into workflow context
   using workflow_set_context with key "child_goal" so the parent workflow can pass it through
   during execution.

4. Check for validation feedback from a previous attempt using workflow_get_output. If validation
   errors are present, fix the issues rather than starting from scratch. Also use
   workflow_get_context with key "validation_result" to see the specific validation output.

5. Design the pipeline to use available workspace agents where appropriate. Use list_agents to
   see what agents are available. Fall back to general-purpose agents for tasks that do not
   match a specialist.

6. Keep the workflow's description concise and specific to the goal.
```

---

This page was generated from [`.stencila/workflows/workflow-create-run/WORKFLOW.md`](https://github.com/stencila/stencila/blob/main/.stencila/workflows/workflow-create-run/WORKFLOW.md).
