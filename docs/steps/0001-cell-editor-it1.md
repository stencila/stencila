# CodeEditor - Iteration I

## Goal

Restructure Sheets and Document to use a shared `CodeEditor` component and improve the architecture of `SheetEditor`.

## Tasks

- Use the same node model in Sheet and Document, so that rendering and manipulation works in the same way
- Turn `MiniLangEditor` into a generalized `CodeEditor` component
- Restructure `SheetEditor` and improve handling of the state for controlling cell editing.

## CodeEditor

Keep this componenent as simple and general as possible.
Disable experimental features

Context:
- `editorSession`: an editorSession with a document, containing a node with a `string` or `text` property

Properties:
- `name`: (optional) used to distinguish surface
- `path`: path to a property containing source code

## Restructure `SheetEditor`

- Sheet maintains an extra tiny document with one cell used for the `FormulaBar` and for the `FormulaEditor`
- Sheet has a state controlling if the `FormulaEditor` is visible
- Setting the selection in either the `FormulaBar` or the `FormulaEditor` activates the edit mode
- Setting the selection in the sheet de-activates the edit mode
- Entering a cell puts the selection into the `FormulaEditor`
- Canceling editing sets the selection into the current anchor cell
- Confirming (`ENTER`) updates the current anchor cell and selects the next cell below the current cell
- When the selection is changed on the sheet while editing a cell, then the last anchor cell is updated
- The `FormulaBar` always displays the content of the current anchor cell
- The `FormulaEditor` is a floating element positioned over the current anchor cell; it is invisible when not in edit mode

**Proposed Structure:**

```
App -> loads document, creates EditorSession)
  - SheetEditor
    - ToolPane
      - Toolbar
      - ExpressionBar
  - ContentPane
      - SheetComponent scrollable canvas for sheet
        - FormulaEditor -> owned by SheetEditor, passed as 'overlay'
      - ContextPane -> e.g. for contextual function documentation, issues panel etc.
  - StatusPane
    - StatusBar
    - WorkflowPane -> e.g. FindAndReplace
```

** Maintaining `edit` state

- Watch for selection changes in both `EditorSession`s, of the sheet, and of the formula:
  - selection has changed in sheet:
    - update the formula (for `FormulaBar`)
    - if in edit mode: de-activate edit mode
  - selection changed in formula:
    - if not in edit mode: activate edito mode
- When entering edit mode, clear the history of the `FormulaEditor`
- Keep the cell id of the anchor cell, so that you know which cell to update when the editor is 'blurred' via selection change
