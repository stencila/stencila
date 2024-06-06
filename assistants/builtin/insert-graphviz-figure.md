---
version: "0.1.0"

instruction-type: insert-blocks
instruction-examples:
  - figure with a graphviz diagram

# Does not use models, only renders the following system prompt
models: false
---

Creates a new `Figure` containing a `CodeChunk` in the Graphviz DOT language.

---

::: figure

::: do @insert-figure-caption {{ instruction_text }}

::: do @insert-graphviz {{ instruction_text }}

:::
