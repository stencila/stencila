---
name: test-compose
description: Test parent workflow composing a child workflow via workflow= sugar
---

```dot
digraph Workflow {
    Start -> Implement

    Implement [workflow="test-compose-child"]
    Implement -> End
}
```
