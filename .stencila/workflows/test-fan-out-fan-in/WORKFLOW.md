---
name: test-fan-out-fan-in
description: Test workflow using parallel fan-out and fan-in
goal: A string containing exactly three color names
---

A simple test of parallel execution using `FanOut` node id for fan-out and a combine node for fan-in.

```dot
digraph Workflow {
    Start -> FanOut

    FanOut  [label="Describe colors in parallel"]
    FanOut -> Red
    FanOut -> Green
    FanOut -> Blue

    Red     [prompt="Reply with ONLY one word: the name of the color with hex code #FF0000"]
    Red -> Combine

    Green   [prompt="Reply with ONLY one word: the name of the color with hex code #00FF00"]
    Green -> Combine

    Blue    [prompt="Reply with ONLY one word: the name of the color with hex code #0000FF"]
    Blue -> Combine

    Combine [prompt="List the three colors from the previous stages as a comma-separated list, e.g. pink, brown, orange. Reply with ONLY the list, nothing else."]
    Combine -> Verify

    Verify  [prompt="Does '$last_output' achieve the goal '$goal'? Reply with ONLY yes or no in lowercase, nothing else."]
    Verify -> End  [condition="context.last_output=yes"]
    Verify -> Fail
}
```
