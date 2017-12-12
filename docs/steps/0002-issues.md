---
title: Issues
author:
  - Oliver Buchtala
type: Feature
status: Draft
---

# Issues

There are two main categories of issues: dynamically evaluated issues and custom issues created by the user.

Issues are bound to a specific cell or to a range of cells.

Issues have a `severity` level: `error`, `warning`, or `info`.

Issues that are bound to a cell are shown in the cell, e.g. underlining the problematic location in the expression, or by showing a coloured triangle.

All Issues are collected in the `IssuesPanel`.

A summary of the number of errors and warnings is displayed in the `StatusBar`.

## Syntax Errors

After a cell expression is changed, the Cell Engine analyses the expression. Any errors detected here are added to the cell's state. A syntax error provides the location in the expression and has severity level `error`.

Example:

```
x = sum(1 2)
```

may result in something like

```
x = sum(1 2)
        ^^^
Syntax Error: expected ','
```

## Runtime Errors

Runtime errors can occur when a cell expression is evaluated.

Examples:

- invalid value types
- unresolved variables
- division by zero

```
x = sum('foo')
```
may result in something like

```
x = sum('foo')
        ^^^^^
Runtime Error: Illegal argument. Expected a number, but received a string.
```

> TODO: Another source of runtime errors is the rendering of a cell's result. For example, a plot specification could have an invalid format

## Tests

> TODO: create a dedicated step for that

## Type Checkers

> TODO: create a dedicated step for that

## Custom Issues

Custom Issues are addressed using [Comments](./0004-comments.md).

# Issues Panel

The Issue Panel is a place where all issues are displayed.
The user can jump to the associated cell or range, and filter issues by type.

> TODO: should there be more filter criteria?

# Statusbar

The Statusbar displays aggregated information for certain issues:

- number of errors and warnings
- number of failed tests

# Iterations

## Iteration I

Tidy-up the implementation, removing experimental and hacky code.

## Iteration II

Observing Cells: being able to observe a range of cells and get notified when a cell has been analysed or evaluated.

## Iteration III

Framework: Issues Manager and Issues Panel should integrate Tests, Type Checkers, and Custom Issues.

> This goes together with first iterations on StEPs 'tests', 'type-checker', and 'comments'.
