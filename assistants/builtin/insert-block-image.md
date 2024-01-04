---
version: "0.1.0"

preference-rank: 100
instruction-type: insert-blocks
instruction-regexes:
  - (?i)\bimage\b

delegates: [none]

transform-nodes: Paragraph
assert-nodes: ^Paragraph$
---

An assistant specialized for inserting a new `Paragraph` containing a single inline `ImageObject`. Intended for when a user wants to insert a block level image that is not encapsulated within a `Figure`.

The user prompt template is rendered to create a `Paragraph` with a single instruction, assigned to `insert-inline-image`, to create an inline `ImageObject`.

---

---

{%% @insert-inline-image {{ instruction_text }} %%}
