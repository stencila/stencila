---
name: test-fan-out-fan-in
description: Test workflow using parallel fan-out and fan-in
---

A simple test of parallel execution using `FanOut` node id for fan-out and a combine node for fan-in.

```dot
digraph Workflow {
    graph [goal="A string containing exactly three color names"]

    Start -> FanOut
    FanOut -> Red
    FanOut -> Green
    FanOut -> Blue
    Red -> Combine
    Green -> Combine
    Blue -> Combine
    Combine -> Verify

    FanOut  [label="Describe colors in parallel"]
    Red     [prompt="Reply with ONLY one word: the name of the color with hex code #FF0000"]
    Green   [prompt="Reply with ONLY one word: the name of the color with hex code #00FF00"]
    Blue    [prompt="Reply with ONLY one word: the name of the color with hex code #0000FF"]

    Combine [prompt="List the three colors from the previous stages as a comma-separated list, e.g. pink, brown, orange. Reply with ONLY the list, nothing else."]

    Verify  [prompt="Does '$last_output' achieve the goal '$goal'? Reply with ONLY yes or no in lowercase, nothing else."]
    Verify -> End  [condition="context.last_output=yes"]
    Verify -> Fail

    Fail
}
```
