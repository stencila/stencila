---
version: "0.1.0"

preference-rank: 150
instruction-type: insert-blocks
instruction-examples:
  - figure with image

delegates: [none]
---

An assistant specialized for inserting a new `Figure` containing a `ImageObject` and a caption.

The system prompt template renders to a `Figure` with new instructions to create a caption (assigned to `insert-figure-caption`) and an image (assigned to `insert-inline-image`) both passed the instruction text.

---

::: figure

// @insert-figure-caption {{ instruction_text }}

{// @insert-inline-image {{ instruction_text }} //}

:::
