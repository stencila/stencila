---
title: Orchestration Tools
description: Internal tools that enable agents to discover other agents and workflows, delegate tasks, and interact with users during execution.
---

The orchestration tools let agents collaborate with each other and with users. They are the mechanism behind Stencila's multi-agent architecture: a manager agent can discover what specialists and workflows are available, route a task to the best match, and ask the user for clarification when needed.

These tools are not part of any provider's default tool set. They are registered separately and only available to agents that explicitly list them in `allowed-tools`. Most users will never call them directly — they are invoked by agents like the built-in [manager agent](#how-these-tools-work-together) as part of Stencila's routing and orchestration layer.

## `ask_user`

Asks the user one or more questions and waits for their responses. Supports freeform text, yes/no, confirmation, single-select, and multi-select question types. When multiple questions are provided, they are presented together as a form where the frontend supports it; otherwise they are presented sequentially.

| Parameter | Type | Required | Description |
| --------- | ---- | :------: | ----------- |
| `questions` | array | ✅ | One or more question objects (see below) |
| `preamble` | string | | Markdown content rendered before the questions as persistent context |

Each question object supports:

| Field | Type | Required | Description |
| ----- | ---- | :------: | ----------- |
| `question` | string | ✅ | The question text to present |
| `type` | string | | `"freeform"` (default), `"yes-no"`, `"confirm"`, `"single-select"`, or `"multi-select"` |
| `options` | array | | Choices for single-select or multi-select questions, each with a `label` and optional `description` |
| `default` | string | | Default answer used when the user skips or times out |
| `header` | string | | Short label displayed above the question |
| `store` | string | | Key under which the answer is stored for use by downstream questions |
| `show_if` | string | | Only present this question when a condition is true (e.g. `"store_key == value"`) |
| `finish_if` | string | | End the interview early if the answer matches this value |

Agents use `ask_user` when they need clarification before proceeding, when a decision point requires human judgment, or as part of a human-in-the-loop workflow where user approval gates the next step.

## `list_agents`

Discovers available agents from all sources — workspace, user configuration, and built-in. Returns a JSON array of agent summaries to help decide which agent to delegate to. Takes no parameters.

Each entry includes:

| Field | Description |
| ----- | ----------- |
| `name` | Agent identifier |
| `description` | What the agent does |
| `source` | Where the agent was found (workspace, user, builtin) |
| `keywords` | Lexical tags for matching against user requests |
| `whenToUse` | Positive signals for when this agent should be selected |
| `whenNotToUse` | Negative signals for when this agent should be avoided |

## `list_workflows`

Discovers available workflows from the workspace's `.stencila/workflows/` directory. Returns a JSON array of workflow summaries to help decide which workflow to delegate to. Takes no parameters.

Each entry includes:

| Field | Description |
| ----- | ----------- |
| `name` | Workflow identifier |
| `description` | What the workflow does |
| `goal` | The workflow's stated goal |
| `path` | Filesystem path to the workflow definition |
| `ephemeral` | Whether this is a temporary workflow from a previous session |
| `keywords` | Lexical tags for matching |
| `whenToUse` | Positive signals for when this workflow should be selected |
| `whenNotToUse` | Negative signals for when this workflow should be avoided |

## `delegate`

Ends the current agent's turn and hands the task to another agent or workflow. The delegatee starts a new session with the provided instruction. Use this after inspecting available agents and workflows with `list_agents` and `list_workflows`.

| Parameter | Type | Required | Description |
| --------- | ---- | :------: | ----------- |
| `kind` | string | ✅ | `"agent"` or `"workflow"` |
| `name` | string | ✅ | Name of the agent or workflow to delegate to |
| `instruction` | string | ✅ | What the delegatee should accomplish — phrased as a task for agents, or a goal for workflows |

The tool validates that the named agent or workflow exists before delegating. If the name is not found, it returns an error so the agent can try a different candidate.

## How these tools work together

The built-in **manager** agent demonstrates the typical orchestration pattern. When a user starts a conversation, the manager:

1. Receives pre-run results from `list_agents` and `list_workflows` — these are automatically executed before the first model call and injected into context to save a round trip.
2. Evaluates the user's request against the discovered agents and workflows using their descriptions, keywords, and when-to-use signals.
3. Uses `ask_user` if the routing decision is ambiguous — for example, when multiple candidates match or the request could be interpreted differently.
4. Calls `delegate` to hand off to the best match with a clear instruction.

This pattern separates routing intelligence from task execution: the manager never does the work itself, it just connects the user to the right specialist. Custom agents can use the same tools to build their own orchestration logic.

> [!info]
> `list_agents` and `list_workflows` are designated as **pre-run tools** — they are executed automatically before the first model call when present in an agent's allowed tools, and their results are injected into the prompt context. This avoids a wasted round trip where the model would otherwise need to call them as its first action.
