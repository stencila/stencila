---
title: "Workflow Record"
description: "Explicit workflow, agent, and definition context."
---

# Workflow Record

Explicit workflow, agent, and definition context.

Workflows describe orchestration around the activity. The fields align with
the Stencila workspace database: `runId` identifies `workflow_runs.run_id`,
`nodeId` identifies `workflow_nodes.node_id` within that run, `artifactId`
identifies `workflow_artifacts.artifact_id`, and definition `contentDigest`
values identify `workflow_definition_snapshots.content_hash`.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`runId`](#run-id) | `string` | No | Workflow run identifier. |
| [`workflowName`](#workflow-name) | `string` | No | Workflow name. |
| [`goalDigest`](#goal-digest) | `string` | No | Digest of the workflow goal or prompt. |
| [`nodeId`](#node-id) | `string` | No | Stencila node identifier associated with the workflow. |
| [`threadId`](#thread-id) | `string` | No | Conversation or workflow thread identifier. |
| [`artifactId`](#artifact-id) | `string` | No | Produced artifact identifier. |
| [`agentSessionId`](#agent-session-id) | `string` | No | Agent session identifier. |
| [`agent`](#agent) | [`AgentRecord`](agent-record) | No | Agent responsible for the workflow. |
| [`definitions`](#definitions) | array<[`DefinitionRecord`](definition-record)> | No | Definitions loaded by the workflow. |

### `runId`

Workflow run identifier.

A run ID lets Stencila and external audit systems correlate the assertion
with `workflow_runs.run_id` without embedding the full workflow log.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `workflowName`

Workflow name.

The name is display metadata and may be redacted or omitted for private
workflows.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `goalDigest`

Digest of the workflow goal or prompt.

The digest gives evidence that a particular private goal existed without
disclosing the prompt text.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `nodeId`

Stencila node identifier associated with the workflow.

This may differ from `document.nodeId` when a workflow operates over a
broader scope than the signed node.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `threadId`

Conversation or workflow thread identifier.

Thread IDs help relate multi-turn agent work while remaining optional for
privacy and for non-conversational workflows.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `artifactId`

Produced artifact identifier.

This is a workflow-system artifact ID, not necessarily the C2PA asset ID.
It is optional because artifacts may be transient or private.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `agentSessionId`

Agent session identifier.

Session IDs are useful for audit trails but can be sensitive, so they are
optional and subject to the privacy policy.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `agent`

Agent responsible for the workflow.

This records the orchestrating agent, while `attributions` records the
role-bearing authorship or responsibility credited to agents.

**Type:** [`AgentRecord`](agent-record) | **Required:** No | **Nullable:** Yes

### `definitions`

Definitions loaded by the workflow.

Definitions are recorded by metadata and digest so their content can stay
private while still being linked to `workflow_definition_snapshots`.

**Type:** array<[`DefinitionRecord`](definition-record)> | **Required:** No


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
