---
title: Cell editor
author:
  - Oliver Buchtala
type: Feature
status: Draft
---

## Introduction

In Stencila there are two different environments with cells, Documents and Sheets, with different requirements regarding UX.

Cells in a Document are reactive, i.e. whenever the content is changed, the cell is re-evaluated (except from paused cells). To avoid noisy feedback and unnecessary recomputations, the update is triggered once the user stops typing.

> This requirement has been pinned down after observing the merits in a workshop environment.
> Moreover, this interaction pattern is consistent with other content, such as paragraphs.
> Reactive re-evaluation avoids the need of an extra gesture for evaluation.

In Spreadsheets the user enters a cell explicitly for editing (`ENTER`), confirming (`ENTER`) or discarding the changes (`ESC`).

> This UX pattern is necessary as a cell has two representations: source and value. Entering a cell opens the source version, leaving the cell flips back to the value.
> This is also the common interaction pattern in other Spreadsheet applications.
> see [#451](https://github.com/stencila/stencila/issues/451) for discussion

## Why not using an existing Code Editor?

The main problem here that existing Code Editor widgets (`CodeMirror`, `ACE`, `Monaco`) use a HTML layout that does not work well in for a word-processor interface, where all content is accessible via cursor navigation. This would require the user to enter the cell first (like in Sheets) and leave

> see [#451](https://github.com/stencila/stencila/issues/451) for discussion

## Features

- Auto-Complete (for variable and function names)
- Contextual Function Documentation
- Cell-Referencing
- Syntax-Highlighting
- Code-Editor sugar as (bracket matching)

## Implementation

### Iteration I

- Goal: a reusable CodeEditor component, used in Sheets as well as in Documents
- [Implementation Instructions](0001-cell-editor-it1.md)

### Iteration II - Code Analyzer

- Goal: generalized API to analyze source code for syntax highlighting and reflection, and integration into `CodeEditor`

> `MiniContext` already provides tokens when analyzing code. It would be good to have the same for other languages.

### Iteration III -  Function Documentation

- Goal: a generalized API to retrieve contextual help for functions, and integration into `CodeEditor`

### Iteration IV - Cell References

- Goal: the ability to select a cell or a range and insert a symbol into the expression.

### Iteration IV - Auto-Completion

- Goal: a generalized API to retrieve suggestions, feeded with data taken from the `CellEngine`
