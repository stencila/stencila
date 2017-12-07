# Log

## 6th Dec 2017

- CodeEditor: We decided that we **really** want a custom CodeEditor for the sake of UX (see [topic](./topics/code-editor.md))

- UX: We decided that we want to stick with the current UX for cells in Documents (direct changes) and Sheets (enter/confirm)

- Reactiveness:
  - in Documents we want to reduce 'noise' by re-evaluating cells only when the user is idle (i.e. not while typing)
  - in Sheets we want to introduce a preview that shows the preliminary evaluation of the cell, corresponding to the value, while the user edits a cell

- IssueManager: we identified the need for two core concepts: '(Dynamic) Issues' (parsing and evaluation errors, checkers, tests, conditional formatting) and 'Comments' (messages attached to a cell or range)

## 4th Dec 2017

We want to have a place where we record decisions, capture ideas that we can link to from the issue tracker