---
title: Generating Diagrams and Code
---

• Let's write a short scientific [document](file:open).

• We're going to write about modeling [The Krebs Cycle](type:0).

• We'll use the ``insert-expand-ideas`` assistant to write a coherent narrative based on a [few ideas](type:1).

• If you press the run button, the assistant will generate a few paragraphs of text, attempting to connect the ideas.

• We can use Graphviz to make create a diagram showing the [reactions and products in the cycle](type:2)

• The diagram generated can be tweaked. For example, the default is often orientation is often left-right (LR). If so, try changing it to TB (top-bottom). Re-run it.

• We can also generate code to plot a [time series](type:3) of concentrations of elements in the Krebs cycle.

---

# The Krebs Cycle

---

::: do @edit-expand-ideas 
::: with

- krebs cycle, background and history
- krebs cycle, the process
- krebs cycle, the significance

:::

---

::: do @insert-graphviz-figure the krebs cycle, showing the sequence of reactions, products and cycle.

---

::: do @insert-code-chunk plot times series of elements for the krebs cycle using scipy

---
