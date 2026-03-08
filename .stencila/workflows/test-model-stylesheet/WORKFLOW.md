---
name: test-model-stylesheet
description: Test model stylesheet application via class and ID selectors
goal: Two numbers
modelStylesheet: |
  * { llm_model: claude-sonnet; }
  .precise { reasoning_effort: high; }
  #Verify { llm_model: gpt-5; }
---

Tests that `modelStylesheet` frontmatter is merged into graph attributes and that universal (`*`), class (`.precise`), and ID (`#Verify`) selectors are applied to nodes. The `.precise` class is assigned via the `class=` node attribute.

```dot
digraph Workflow {
    Start -> CountA -> CountB -> Verify

    CountA [prompt="Reply with ONLY the number: 1", class="precise"]
    CountB [prompt="Reply with ONLY the number: 2", class="precise"]
    Verify [prompt="Are the outputs from the previous stages numbers 1 and 2? Reply with ONLY yes or no in lowercase."]

    Verify -> End  [condition="context.last_output=yes"]
    Verify -> Fail

    Fail
}
```
