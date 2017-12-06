# Cell Editor

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

## Iteration I - Shared CodeEditor

Use the same node model in Sheet and Document, so that rendering and manipulation works in the same way

### CodeEditor

An implementation that should be as general as possible,
so that we can re-use it for other purposes, later

Props:
  - `editorSession`: an editorSession with a document, containing a node with a `string` or `text` property
  - `path`: path to a property containing source code


### Embedding into Document

- `DocumentCellComponent`, which extends `IsolatedNodeComponent`, renders a `CodeEditor` instance providing the real editorSession and cell


### Embedding into Sheet

- Sheet maintains a stub instance of a sheet document, with one cell
- There are two components with a `CodeEditor` instance for the very same
  cell of the stub-sheet: the `ExpressionBar` and the `SheetCellEditor`
- Sheet has a state controlling if the cell-editor is visible (`cell-edit` mode)
- Focusing the ExpressionBar or entering the cell-editor activates the `cell-edit` mode
- Confirming, cancelling, or blurring the editor de-activates the `cell-edit` mode
- ExpressionBar:
  - if **not** in `cell-edit` mode: render the source of the currently selected (anchor) cell
  - if in `cell-edit` mode: render the CodeEditor
- CellEditor:
  - is a floating element that is positioned over the currently selected cell
  - if **not** in `cell-edit` mode: render CodeEditor as invisible
  - if in `cell-edit` mode: render CodeEditor as visible and positioned over the current cell
- when entering `cell-edit` mode: reset the history of the stub editor session and set the content of the stub-cell to the content of the original cell
- when confirming (enter): write the content into the original cell and select the next cell
- when leaving (enter or blur): write the content into the original cell
- when cancelling: clear the content of the stub cell and on cancel the editor is closed without changes

#### Notes from Reviewing the current implementation

We need to refactor a bit.

SheetApp:
- gets EditorSession for a SheetDocument
- creates (transient) EditorSession for cell editing: 'transientEditorSession'
- layout:
  - ToolPane (?)
    - SheetToolbar (-> regular Toolbar)
    - ExpressionBar
  - ContentPane
      - SheetPane: scrollable canvas for sheet (currently SheetComponent)
      - ContextPane: e.g. to show contextual function documentation, issues panel etc.
      - "FloatingCellEditor"
  - StatusPane:
    - StatusBar: a bar at the bottom, e.g. shows number of errors, etc.
    - WorkflowPanes: e.g. FindAndReplace
- state:
  - `cellEditing`: 'is cursor in cell editor'
    - `true` => FloatingCellEditor is visible (and positioned)
    - `false` => FloatingCellEditor is hidden

- Determining `cellEditing` state:
  - try to null selection in 'transientEditorSession' when closing 'FloatingCellEditor'
  - with that, whenever selections changes to not null => `cellEditing = true`


## Iteration II - Code Analyzer

API to analyze source code for syntax highlighting and reflection.

`MiniContext` already provides tokens when analyzing code. It would be good to have the same for other languages.

## Iteration III -  Function Documentation

An API to retrieve contextual help for functions.

## Iteration IV - Cell References

A mode allowing to select a cell or a range used to insert a symbol into the expression.

## Iteration IV - Auto-Completion

An API to retrieve suggestions for the current content.
