---
# Test `insert-code-chunk` makes use of variables in the kernel.
---

```python exec
# Use exec on an external file to load the local variables into the kernel
# This forces the assistant to rely on the variable hints, rather than reading the code.
with open('variables.py') as f:
  exec(f.read(), locals())
```

::: do @insert-code-chunk compute something using all the local variables you know of

::: do @insert-code-chunk assess what shaped rocket can best escape mars.
