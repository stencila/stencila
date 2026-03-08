---
name: test-subgraph-defaults
description: Test subgraph scoping of node default attributes
goal: Three words from two phases
---

Tests that `subgraph` blocks scope `node [...]` default attributes (such as `class`) to nodes declared within them. Nodes inside each subgraph inherit the subgraph's defaults unless they explicitly override them.

```dot
digraph Workflow {
    Start -> Alpha -> Beta -> Combine -> Verify

    subgraph cluster_phase1 {
        label = "Phase 1"
        node [class="fast"]

        Alpha [prompt="Reply with ONLY the word: alpha"]
    }

    subgraph cluster_phase2 {
        label = "Phase 2"
        node [class="fast"]

        Beta [prompt="Reply with ONLY the word: beta"]
    }

    Combine [prompt="List the two words from previous stages separated by a comma. Reply with ONLY the list."]

    Verify [prompt="Does '$last_output' contain both 'alpha' and 'beta'? Reply with ONLY yes or no in lowercase, nothing else."]
    Verify -> End  [condition="context.last_output=yes"]
    Verify -> Fail

    Fail
}
```
