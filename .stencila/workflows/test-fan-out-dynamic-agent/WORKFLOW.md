---
name: test-fan-out-dynamic-agent
description: Test dynamic parallel fan-out over a runtime list produced by an agent via workflow_set_context
---

Tests the `fan-out` attribute for dynamic parallel fan-out where the list is produced by an agent node calling `workflow_set_context` rather than by a shell node with `store`. The agent is prompted to write a three-element JSON array to context key `items` using the `workflow_set_context` tool. The fan-out node reads that array and spawns one branch per item. Each branch runs a shell node that echoes the item, verifying per-item context injection (`$fan_out.item`, `$fan_out.index`, `$fan_out.total`). The fan-in node collects results. A final shell node echoes `$parallel.outputs` to verify the aggregated output.

```dot
digraph Workflow {
  Start -> Seed

  Seed [
    prompt-ref="#seed-prompt",
    context-writable=true
  ]
  Seed -> FanOut

  FanOut [fan-out="items"]
  FanOut -> Process

  Process [shell="printf '%s:%s/%s' '$fan_out.item' '$fan_out.index' '$fan_out.total'"]
  Process -> FanIn

  FanIn
  FanIn -> Verify

  Verify [shell="echo '$parallel.outputs'"]
  Verify -> End
}
```

```text #seed-prompt
You must call the workflow_set_context tool exactly once to store a JSON array with key "items" and value ["alpha","beta","gamma"]. Do not output anything else.
```
