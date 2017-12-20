---
author: Oliver Buchtala
assignees: Oliver Buchtala
PR: https://github.com/stencila/stencila/pull/463
---
# Issues - Iteration I

## Goal

Tidy-up the implementation, removing experimental and hacky code.

## Tasks

- Disable and remove all experimental implementations (row/column aggregations, tests) in the  `IssueManager`.
- Use the `IssueManager` to maintain Syntax and Runtime Errors instead of setting errors in the `Engine` directly
- Let `IssueManager` maintain a set of all current issues, and  aggregate counters for different severity levels.
- Display all issues in the `IssuesPanel` allowing to jump to the specific cell.
- Display number of errors and warnings in the Statusbar.
