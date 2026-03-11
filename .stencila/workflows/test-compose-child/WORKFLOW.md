---
name: test-compose-child
description: Child workflow used by the workflow composition integration test
---

```dot
digraph Workflow {
    Start -> Echo1

    Echo1 [cmd="echo child-1"]
    Echo1 -> Echo2

    Echo2 [cmd="echo child-2"]
    Echo2 -> End
}
```
