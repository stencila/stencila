---
version: "0.1.0"

preference-rank: 100
instruction-type: insert-blocks
instruction-examples:
  - insert an inline image
  - insert a inline picture
delegates: [none]
---

An assistant specialized for inserting a new `Paragraph` containing a single inline `ImageObject`. Intended for when a user wants to insert a block level image that is not encapsulated within a `Figure`.

The system prompt template renders to a `Paragraph` with a new instruction, assigned to `insert-inline-image`, to create an inline `ImageObject` following the instruction text.

---

{// @insert-inline-image {{ instruction_text }} //}
