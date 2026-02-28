---
name: test-count-to-three
description: Counts from one to three
---

A simple test of $last_output and $last_stage expansion.

```dot
digraph CountToThree {
    graph [goal="Count from one to three"]

    Start -> One -> Two -> Three -> Verify
    Verify -> End  [condition="context.last_output=yes"]
    Verify -> Fail

    One    [prompt="Reply with just the number: 1"]
    Two    [prompt="Add one to $last_output and reply with just the result."]
    Three  [prompt="Add one to $last_output and reply with just the result."]

    Verify [prompt="Does $last_output equal 3? Reply with ONLY yes or no in lowercase, nothing else."]
    Fail
}
```
