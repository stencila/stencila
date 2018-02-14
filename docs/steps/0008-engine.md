---
title: Engine
author:
  - Oliver Buchtala
type: Refactor
status: Draft
---

## Introduction

The current Engine implementation has several flaws making a redesign
inevitable:

- the Engine is run in the rendering thread, blocking the UI on during computations
- the Engine has an insufficient model for error states
- there is no concept for cyclic dependencies and ambigious variable declarations
- the Engine is difficult to test because of the asynchronous, data driven nature of the implementation

This StEP introduces the third generation of the architectural design for Stencila's Engine. Which overcomes existing problems by

- choosing a better metaphor for the system,
- separating problems with an asynchronous vs. a synchronous nature,
- and modularizing the implementation so that it can be run isolated from the rest of the application (e.g. using a web worker).

## Concepts

### Cell Graph

In previous implementations the cell graph was implemented in a conventional way, maintaining a graph data structure with nodes and edges was maintained. However, the graph methaphor does not reflect the automatic, associative nature of system of cells where links are formulated symbolically.

A better metaphor is a chemical reaction, where molecules are mixed together, automatically emerging into a reaction system following a set of rules. Another metaphor is a trading system, where brokers sell and buy products, enforcing a monopol for each product, and disallowing loops.

Describing the system as a set of cells, with desires and offers, instead of graph a with links between cells, provides a better representation of the problem's nature. Problematic scenarios, such as unresolvable inputs, ambigious variable declarations, or cyclic dependencies are addressed inherently, and not just treated as system boundaries.

The cell graph itself works deterministically, like an automaton. For that a system of cell states is introduced, formalizing the life cycle of a cell. Structural changes and updates are executed instantaneously. Separating code evaluation from the inference on the cell graph allows to test critical parts of the system independently.

> TODO: provide some examples to illustrate the concept

## Cell States

- `BROKEN`: the cell has a problem within the current system, that prevents further evaluation.
- `FAILED`: the cell has been evaluated but there was a error during evaluation (runtime and validation errors)
- `BLOCKED`: one of the cell's inputs is `BROKEN`, has `FAILED`, or is `BLOCKED`
- `WAITING`: but at least one is not yet `OK`
- `READY`: the cell is ready for evaluation
- `RUNNING`: evaluation has been started
- `OK`: evaluation succeeded, i.e. the cell's value is up-to-date


## Implementation

- [Iteration I](0006-engine-it1.md): Cell Graph
