---
name: test-goal-gates
description: Test goal gate enforcement and retry_target loopback
retryTarget: Attempt
---

Tests that `goal_gate=true` prevents the pipeline from exiting until the gated node has succeeded. The `Attempt` shell node uses a counter file to fail deterministically on its first execution and succeed on its second. When `Attempt` fails, the fail edge routes to `End`, where the unsatisfied goal gate triggers a `retryTarget` loopback to `Attempt`. On the retry, the counter has been incremented so the command succeeds, satisfying the goal gate and completing the pipeline.

```dot
digraph Workflow {
    Start -> Setup -> Attempt -> End
    Attempt -> End [condition="outcome=fail"]

    Setup   [cmd="echo 0 > /tmp/stencila-test-goal-gate.txt"]
    Attempt [cmd="COUNT=$(cat /tmp/stencila-test-goal-gate.txt); COUNT=$((COUNT+1)); echo $COUNT > /tmp/stencila-test-goal-gate.txt; test $COUNT -ge 2", goal-gate=true]
}
```
