---
name: test-overrides
description: Test overrides application via class and ID selectors
goal: Two numbers
overrides: |
  * { model: claude-sonnet; }
  .precise { reasoning_effort: high; }
  #Verify { model: gpt-5; }
---

Tests that `overrides` frontmatter is merged into graph attributes and that universal (`*`), class (`.precise`), and ID (`#Verify`) selectors are applied to nodes. The `.precise` class is assigned via the `class=` node attribute.

```dot
digraph Workflow {
  Start -> CountA

  CountA [prompt="Reply with ONLY the number: 1", class="precise"]
  CountA -> CountB

  CountB [prompt="Reply with ONLY the number: 2", class="precise"]
  CountB -> Verify

  Verify [prompt="Are the outputs from the previous stages numbers 1 and 2? Reply with ONLY yes or no in lowercase."]
  Verify -> End  [condition="context.last_output=yes"]
  Verify -> Fail

  Fail
}
```
