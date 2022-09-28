# Simple

::: for item in [1,2,3]

This paragraph will be repeated with a different value for `item`{exec}.

:::

# Nested

For loops can be nested,

::: for row in [['a', 'b'], ['c', 'd']]

::: for col in row

This paragraph will be repeated for each column and row: `col`{exec}

:::

:::

# Else

For loops can have an `else` clause which specifies the content to be rendered if there are not items,

::: for row in []

This content is never shown

::: else

There are no items

:::

# Deviations

Here is a test that missing content will still be parsed as a `ForEach`,

::: for item in [1,2,3]

::: else

:::

However note that there must be empty lines between sections. e.g this should _not_ be parsed as a `ForEach`,

::: for item in [1,2,3]
::: else
:::

# Regressions

Some Markdown that did not parse correctly during prop tests

## Table in `otherwise`

::: for item in text {py}

Content paragraph

::: else

Otherwise paragraph

| A   | B   |
| --- | --- |
| 1   | 2   |

:::
