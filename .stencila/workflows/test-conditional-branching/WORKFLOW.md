---
name: test-conditional-branching
description: Test conditional diamond node routing based on LLM outcome
goal: Produce a valid three-letter word
---

Tests conditional routing via a `Check*` node ID (sugar for `shape=diamond`). The `CheckValid` node is a no-op conditional that routes based on `outcome=success` / `outcome!=success` edge conditions. On failure, an edge loops back to retry generation.

```dot
digraph Workflow {
    Start -> Generate

    Generate [prompt="Reply with ONLY a single three-letter English word in lowercase, nothing else."]
    Generate -> CheckValid

    CheckValid [label="Check valid"]
    CheckValid -> Accept   [label="Valid", condition="outcome=success"]
    CheckValid -> Generate [label="Invalid", condition="outcome!=success"]

    Accept   [prompt="Reply with ONLY the word: $last_output"]
    Accept -> Verify

    Verify [prompt="Is '$last_output' a single three-letter English word in lowercase? Reply with ONLY yes or no in lowercase, nothing else."]
    Verify -> End  [condition="context.last_output=yes"]
    Verify -> Fail
}
```
