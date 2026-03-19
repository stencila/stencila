---
name: test-compose-gate
description: Parent workflow that invokes a child workflow containing a human gate
---

```dot
digraph Workflow {
  Start -> RunChild

  RunChild [workflow="test-child-gate"]
  RunChild -> End
}
```
