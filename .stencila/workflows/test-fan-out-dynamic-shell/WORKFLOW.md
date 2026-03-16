---
name: test-fan-out-dynamic-shell
description: Test dynamic parallel fan-out over a runtime list using shell nodes only
---

Tests the `fan_out` attribute for dynamic parallel fan-out. A shell node produces a JSON array and stores it in context using `store`/`store_as`. The fan-out node reads that array and spawns one branch per item. Each branch runs a shell node that echoes the item, verifying per-item context injection (`$fan_out.item`, `$fan_out.index`, `$fan_out.total`). The fan-in node collects results. A final shell node verifies that `$parallel.outputs` contains the expected values. No LLM calls or API keys are required.

```dot
digraph Workflow {
  Start -> Seed

  Seed [shell="echo '[\"alpha\",\"beta\",\"gamma\"]'", store="items"]
  Seed -> FanOut

  FanOut [fan_out="items"]
  FanOut -> Process

  Process [shell="printf '%s:%s/%s' '$fan_out.item' '$fan_out.index' '$fan_out.total'"]
  Process -> FanIn

  FanIn
  FanIn -> Verify

  Verify [shell="echo '$parallel.outputs'"]
  Verify -> End
}
```
