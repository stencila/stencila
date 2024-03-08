---
# We exec an external file to get the local variables
# This forces stencila to rely on the hints, rather than reading the code.
---
```python exec
with open('local_defs.py') as f:
  exec(f.read(), locals())
```

// @insert-code-chunk compute something using all the local variables you know of
