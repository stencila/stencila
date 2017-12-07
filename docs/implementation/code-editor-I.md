# Instructions for Implementing `CodeEditor - Iteration I`

Use the same node model in Sheet and Document, so that rendering and manipulation works in the same way

## CodeEditor

A Component that is as general as possible, so that we can re-use it for other purposes later

Props:
  - `editorSession`: an editorSession with a document, containing a node with a `string` or `text` property
  - `path`: path to a property containing source code

## Embedding into Document

- Use a Component extending `IsolatedNodeComponent`
- embed a `CodeEditor` providing the real editorSession and cell

## Embedding into Sheet

- Sheet maintains a stub instance of a sheet document, with one cell
- There are two components with a `CodeEditor` instance for the very same
  cell of the stub-sheet: for the expression bar and the cell-editor
- Sheet has a state controlling if the cell-editor is visible
- Focusing the ExpressionBar or entering the cell-editor activates the edit mode
- Confirming, canceling, or blurring the editor deactivates the edit mode
- Expression-Bar: should render the source of the currently selected (anchor) cell
- Cell-Editor:
  - is a floating element that is positioned over the currently selected cell
  - if not in edit mode: render as invisible, otherwise visible and position over the current (anchor) cell
- when entering edit mode: reset the history of the stub editor session and set the content of the stub-cell to the content of the original cell
- when confirming (enter): write the content into the original cell and select the next cell
- when leaving (enter or blur): write the content into the original cell
- when canceling: clear the content of the stub cell and on cancel the editor is closed without changes

### Notes from Reviewing the current implementation

We need to re-factor a bit.

- for Sheets try to create a structure like this (feel free to change naming)
```
SheetApp:
- gets EditorSession for a SheetDocument
- creates a staging EditorSession for cell editing (with a sheet document with only one cell)
- layout:
  - ToolPane (?)
    - SheetToolbar (-> regular `Toolbar`)
    - ExpressionBar
  - ContentPane
      - SheetPane: scrollable canvas for sheet (currently SheetComponent)
      - ContextPane: e.g. to show contextual function documentation, issues panel etc.
      - "FloatingCellEditor"
  - StatusPane:
    - StatusBar: a bar at the bottom, e.g. shows number of errors, etc.
    - WorkflowPanes: e.g. FindAndReplace
- state:
  - `cellEditing`: ~ 'is cursor in cell editor'
    - `true` => FloatingCellEditor is visible (and positioned)
    - `false` => FloatingCellEditor is hidden
```
- Derive the `cellEditing` state from the selection:
  - either it changes on the 'real' session, then quit cell-editing
  - or it changes in the 'staging' session, then start / continue cell-editing
  - set selection in the 'staging' session to null when closing (confirm, cancel, blur)