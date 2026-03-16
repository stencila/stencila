---
name: test-context-conditions
description: Test edge conditions that reference context.* variables for multi-way routing
---

Tests multi-way conditional routing using `context.*` keys in edge conditions. The `Classify` shell node writes a known value to `context.shell.output`, and the `CheckType` conditional node routes to different handlers based on that value. Only the `HandleA` branch leads to `End`; all other branches lead to `Fail`, proving that incorrect routing is caught. Because every node uses deterministic shell commands, no LLM calls or API keys are required.

```dot
digraph Workflow {
  Start -> Classify

  Classify [shell="printf type-a"]
  Classify -> CheckType

  CheckType [branch="Check type"]
  CheckType -> HandleA [condition="context.shell.output=type-a"]
  CheckType -> HandleB [condition="context.shell.output=type-b"]
  CheckType -> HandleDefault

  HandleA [shell="printf handled-a"]
  HandleA -> End

  HandleB [shell="printf handled-b"]
  HandleB -> Fail

  HandleDefault [shell="printf handled-default"]
  HandleDefault -> Fail

  Fail
}
```
