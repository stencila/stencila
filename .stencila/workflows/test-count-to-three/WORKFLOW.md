---
name: test-count-to-three
description: Test workflow that counts from one to three
---

A simple test of `$last_output` and `$goal` expansion.

```dot
digraph Workflow {
    graph [goal="3"]

    Start -> One -> Two -> Three -> Verify

    One    [prompt="Reply with just the number: 1"]
    Two    [prompt="Add one to $last_output and reply with just the result."]
    Three  [prompt="Add one to $last_output and reply with just the result."]

    Verify [prompt="Does $last_output equal $goal? Reply with ONLY yes or no in lowercase, nothing else."]
    Verify -> End  [condition="context.last_output=yes"]
    Verify -> Fail

    Fail
}
```
