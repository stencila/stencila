An insert instruction with no assignee:

::: do insert a 4x10 table

An insert instruction with an assignee:

::: do @insert-table top five highest mountains, with height in metres

An insert instruction with an assignee but a blank instruction

::: do @insert-code-chunk 

An edit instruction (has content) with no assignee:

::: do improve this paragraph
::: with

The paragraph to be improved.

:::

An edit instruction with an assignee:

::: do @anne please improve this paragraph
::: with

Another paragraph to be improved.

:::

An insert instruction with an assignee and a suggestion:

::: do @insert-code-chunk analyze data

::: insert

```exec
some code
```

:::

An edit instruction with a suggestion:

::: do more succinct
::: with

A rather long winded paragraph.

:::

::: replace
::: with

A paragraph.

:::