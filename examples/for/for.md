A simple for loop

::: for item in [1,2,3]

This paragraph will be repeated with a different value for `item`{exec}

:::

With a programming language specified

::: for item in [1,2,3]{python}

This content is repeated

:::

Nested for loops

::: for outer in [['a', 'b'], ['c', 'd']]

::::: for inner in outer

This paragraph will be repeated for each inner item `inner`{exec}

:::::

:::

With an else clause

::: for item in []

This content is never shown

::: else

There are no items

:::

With no content in 'content' or 'otherwise'

::: for item in []

::: else

:::