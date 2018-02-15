# Issues - Iteration III

Goal: being able to create tests that are run automatically, to inspect the test results on one panel, and indicate failed tests in the status-bar and the cell itself.

Tasks:
- Extend the schema for 'Test'
- Introduce a `TestsManager`
- Introduce a `TestsPanel`

> TODO: the type-checker could be implemented as a test, too

## Introduce a `<test>` element

A test has a set of cell it is applied to and an assertion the consists of a message and an expression.

In XML this looks like:

```
<tests>
  <test>
    <cells>A1:B10</cells>
    <assertion message="Value should be greater zero" test="value > 0" />
  </test>
</tests>
```

## `TestManager`

The `TestManager` runs test when cells have been evaluated. For that it maintains an observer for every test.
The `TestManager` updates the registration, whenever a test is added, removed, or the cell range is changed. For that it listens to document changes.


### Initialization

- register all existing tests
- start listening for document changes

```
let tests = getTests(sheet)
tests.forEach((test) => {
  let observer = this.registerObserver(test)
  this.registrations[test.id] = observer
})

this.listenToDocumentChanges()
```

### Registration

- use the new Engine cellRange observer API
- register observer for 'evaluated'
- run test for cell when triggered

```
registerObserver(test) {
  let engine = this.getEngine()
  let cellRange = getCellRange(test)
  return engine.observeCellRange(cellRange, {
    evaluated(cell) {
      this.runTest(test, cell)
    }
  })
}
```

### Update Registration

- register an observer for when a test is added
- remove and dispose the observer when a test is removed
- remove the old observer and add a new one when the cell range of a test has been changed

```
// when created
this.registerObserver(test)

// when removed
this.registrations[test.id].dispose()
delete this.registrations[test.id]

// when changed
this.registrations[test.id].dispose()
delete this.registrations[test.id]
this.registerObserver(test)
```

### Running a test

- take all attributes from the test (language, condition, message)
- take the value from the cell
- evaluate the condition with the value exposed
- when ok remove the issue associated with the test
  otherwise add an issue

```
runTest(test, cell) {
  let issueManager = this.getIssueManager()
  let engine = this.getEngine()
  let language = getLanguage(test)
  let expression = getCondition(test)
  let message = getMessage(test)
  let value = getValue(cell)
  let success = engine.evaluateExpression(language, expression, { value })
  if (success) {
    issueManager.removeIssue(cell, test.id)
  } else {
    issueManager.addIssue(cell, test.id, {
      severity,
      message
    })
  }
}
```

## `TestsPanel`

Add a panel displaying all tests and the result.

A new test can be added and existing tests can be removed.
For each test there is
- one text field for a cell or range expression,
- and one assertion with
  - a text field for the message
  - and a `CodeEditor` for the expression
