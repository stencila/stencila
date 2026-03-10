---
name: test-edge-weights
description: Test edge weight-based routing priority
---

Tests that the engine selects the higher-weight edge when multiple unconditional edges leave a node. The `Gate` node has two outgoing edges with different weights; the edge with `weight=10` should always win over `weight=1`. If the engine correctly picks the heavier edge, `PathA` runs and the pipeline reaches `End`. If it incorrectly picks the lighter edge, `PathB` runs and the pipeline hits `Fail`.

```dot
digraph Workflow {
    Start -> Gate

    Gate [cmd="echo gate"]
    Gate -> PathA [weight=10]
    Gate -> PathB [weight=1]

    PathA [cmd="echo alpha"]
    PathA -> End

    PathB [cmd="echo beta"]
    PathB -> Fail
}
```
