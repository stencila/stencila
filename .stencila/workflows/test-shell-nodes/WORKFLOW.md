---
name: test-shell-nodes
description: Test shell command execution nodes using cmd= and shell= sugar
---

Tests the `cmd=` and `shell=` property shortcuts, which expand to `shape=parallelogram` and the `shell` handler type. This workflow has no LLM calls — every task node runs a shell command. The pipeline creates a temporary file, reads it back, and cleans up.

```dot
digraph Workflow {
  Start -> CreateFile

  CreateFile [cmd="echo hello > /tmp/stencila-test-shell-nodes.txt"]
  CreateFile -> ReadFile

  ReadFile [shell="cat /tmp/stencila-test-shell-nodes.txt"]
  ReadFile -> Verify

  Verify [shell="echo '$last_output' | grep -qx hello && echo yes || echo no"]
  Verify -> Cleanup  [condition="context.last_output=yes"]
  Verify -> CleanupFail

  Cleanup [cmd="rm -f /tmp/stencila-test-shell-nodes.txt"]
  Cleanup -> End

  CleanupFail [cmd="rm -f /tmp/stencila-test-shell-nodes.txt"]
  CleanupFail -> Fail

  Fail
}
```
