---
title: Cell editor
author:
  - Oliver Buchtala
type: Feature
status: Draft
---

# Introduction

In Stencila there are two different environments with cells, Documents and Sheets, with different requirements regarding UX.

**Document Cells**

In a Document, cells are edited in a reactive manner, i.e. whenever the content is changed, the cell is re-evaluated. To avoid too early feedback, re-evaluation is triggered once the user stops typing.

This interaction pattern is consistent with other content in a Document, such as paragraphs, i.e. every atomic change has an immediate effect.

> Note: reactive evaluation avoids the need of an extra gesture for (re-)evaluation

**Sheet Cells**

In a Sheet, the user enters a cell explicitly for editing (`ENTER`) or uses the `FormulaBar`. A change is confirmed explicitly (`ENTER`) or discarded (`ESC`).

This interaction pattern is necessary as a cell has two representations: source and value. Entering a cell for editing opens the source version, leaving the cell flips back to the value representation.

While editing a preview is evluated in a reactive manner, but the entire Sheet is updated only after confirming the change.

# Concepts

## Function Usage Help

> TODO: we should move this into a dedicated StEP

When using functions the user is assisted by showing a contextual help
describing how to use a function. When the cursor is over a part of the expression belonging to a function call a short information is shown in a pop-up.

```
= su|m(1, 2, 3)
    ^
-------------------------------
| sum(value, [...values]) [ℹ] |
-------------------------------
```

To assist the user with providing the function arguments, the current argument is hightlighted when the cursor is at the corresponding position.  

```
= sum(1|)
       ^
-------------------------------
| sum(value, [...values]) [ℹ] |
|     ^^^^^                   |
-------------------------------
```

A detailed description of the function can be opend via a button `[ℹ]`, which is then shown side-by-side in the `Context Panel`.

## Cell References

> TODO: we should move this into a dedicated StEP

If the cursor is not on a cell reference, a new symbol is inserted.

```
= 1 + |
```
After clicking on `A1`

```
= 1 + A1|
```

If the cursor is on a cell reference this symbol is replaced, otherwise a new symbol is inserted. I.e. in the previous case the user clicks on cell `A2`, the symbol `A1` is replaced.

```
= 1 + A2|
```

Similar a symbol can be updated at a later following the same rule.

```
= 1 + A2| + B2
```

If the user clicks on `A3` this would lead to

```
= 1 + A3| + B2
```

## Auto-Completion

> TODO

## Basic Code Editing

In a professional Code Editor there is a lot of functionality, such as syntax-highlighting, bracket matching, macros, etc. Stencila's cell editor has only limited capabilities in this regard as cells are meant for small chunks of code, or single expressions.

# Distinction

**Why not using an existing Code Editor?**

The main problem here that existing Code Editor widgets (`CodeMirror`, `ACE`, `Monaco`) use a HTML layout that does not work well in for a word-processor interface, where all content is accessible via cursor navigation. This would require the user to enter the cell first (like in Sheets) and leave

> see [#451](https://github.com/stencila/stencila/issues/451) for discussion

**Function Usage Help**

In Google Spreadsheets the detailed function usage description is shown by default, but can be collapsed. As the pop-up is opened automatically, this has the disadvantage that a lot of content is obscured. In Stencila the default is to show only a short version in the pop-up, and let the user open the detailed description explicitly if they demand.

**Cell References**

In Google Spreadsheets the behavior of how cell references are inserted is different if the user is writing a new expression or changing an existing one.
While writing an expression from scratch, a symbol is inserted one selecting another a cell or range. Selecting a different cell updates the symbol, if the user has not typed in the meantime. After adding an operator or a comma, a new cell reference can be added. In other cases clicking on a cell blurs the editor, especially a cell reference can not be changed at a later time. This behavior not consistent, and thus difficult to grasp.

In Microsoft Excel a cell reference is always inserted, in most cases preceded with a `+`, except after a comma or a `+`.

In LibreOffice Calc a cell reference is always inserted, without preceding character.

Stencila tries to provide a better UX with a hybrid approach, avoiding the inconsistencies of the approach in Google Spreadsheets.

## Implementation

### Iteration I

- Goal: a reusable CodeEditor component, used in Sheets as well as in Documents
- [Implementation Instructions](0001-cell-editor-it1.md)

### Iteration II - Code Analysis

- Goal: generalized API as a preparation for `Function Usage` and `Cell References`

> `MiniContext` already provides tokens when analyzing code. It would be good to have the same for other languages.

### Iteration III -  Function Usage

- Goal: a generalized API to retrieve contextual help for functions, and integration into `CodeEditor`

### Iteration IV - Cell References

- Goal: the ability to select a cell or a range and insert a symbol into the expression.

### Iteration IV - Auto-Completion

- Goal: a generalized API to retrieve suggestions, feeded with data taken from the `CellEngine`
