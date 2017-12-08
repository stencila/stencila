# Issues - Iteration I

Goal: tidy-up the implementation

Tasks:
- Disable and remove all experimental implementations (row/column aggregations, tests) in the  `IssueManager`.
- Use the `IssueManager` to maintain Syntax and Runtime Errors instead of setting errors in the `Engine`
- Aggregate only counters for issues ordered by severity level.

> TODO: