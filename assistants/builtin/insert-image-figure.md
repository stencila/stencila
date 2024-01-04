---
version: "0.1.0"

preference-rank: 150
instruction-type: insert-blocks
instruction-regexes:
  - (?i)\bfigure with image\b

delegates: [none]

transform-nodes: Figure
assert-nodes: ^Figure$
---

An assistant specialized for inserting a new `Figure` containing a `ImageObject` and a caption.

The user prompt template is rendered to create a `Figure` with instructions to create a caption and an image (assigned to the `insert-figure-caption` and `insert-inline-image` assistants respectively) passed the instruction text.

---

---

::: figure

%% @insert-figure-caption {{ instruction_text }}

::>

{%% @insert-inline-image {{ instruction_text }} %%}

:::
