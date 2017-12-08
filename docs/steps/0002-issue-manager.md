title: Issue manager
author:
  - Oliver Buchtala
type: Feature
status: Draft
---

## What we have so far

- Issues are messages bound to a cell.
- The `IssueManager` provides an API to access and create issues.
- Every issue has a severity level: `error`, `warning`, or `info`.
- There are different types of issues:
  - `SyntaxError`:
    - coming from the engine when parsing a cell expression
    - has severity `error`
  - `RuntimeError`:
    - coming from dynamic execution of a cell
    - has severity `error`
  - `TestResult`:
    - a failed test has severity 'error'
    - a successful test has severity 'info'
- Issues are rendered inside a cell (red-triangle)
- StatusBar displays aggregated information (number of errors and warnings)
- IssuePanel displays all issues in a list
- Storing issues on the cell state
  - easy to render inside a cell in a context independent way
  - harder to retrieve a summary for the IssuesPanel or the StatusBar

## Requirements

- The issues panel is a central and important concept - so good to get it's API right and make it easy for other to contribute checks. For examples of checks that may be added in the future see the "Best practices check" section of https://f1000research.com/articles/3-6/v2
- The issue markers on the column and row headers are a little confusing. They are intended to indicate there is an issue somewhere on the row/column but suggest that there is a problem with the entire column/row. Maybe drop these and instead provide:
  - auto-scrolling to the relevant cell when the issue is clicked
  - ability to filter issues by column/row/type (including counts)
- Given that ad-hoc formatting will not be allowed, people want the ability to mark cells as "suspicious", "needs attention", or add some other annotation (the sort of thing that they usually use ad-hoc fromatting for). This could be a part of the issue panel: a user could create a custom issue e.g. "Machine not calibrated", "Needs checking" etc and then use an "apply issue" tool to click on cells and apply the issue to them.
- This is a somewhat different topic, but related to how test fails are recorded in the issue panel. Instead of having separate test cells it would probably be better to have tests as cell attributes. For example, currently, if I had 10 cols x 1000 rows of cells that I wanted to apply the same test to, I would have to create 10 cols x 1000 rows of test cells. Instead, if tests were an attribute of cells, then I could create a test and apply it to all those cells. This is also related to having tests defined to each type e.g. a "human height" type might have tests to check value greater than 0 and less then 3m (ie. types have tests and cells can have additional custom tests).

## Open Questions

We need to identify concepts and clarify if 'Issues' are the right concept for all of the requirements.

1. Syntax-/Runtime-Errors during the evaluation process
2. Checkers/Tests that run whenever a cell has been evaluated
3. Tags created to highlight a cell (can be created on a range for convenience)
4. Comments/markers (with ability to link to a cell/range)

`1.` and `2.` are creating 'issues' dynamically at different stages of a cell's life-cycle
`3.` and `4.` are different concepts that may be used to communicate 'custom issues'
`2.` and `4.` are kind of 'floating', i.e. not part of the table of cells
`2.` could be used for dynamic formatting
