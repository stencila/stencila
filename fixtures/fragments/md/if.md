# Simple

::: if true

This paragraph should be shown.

:::

# Else

An `else` clause specifies the content to be rendered if the condition is not truthy,

::: if false

This paragraph should NOT be shown.

::: else

This paragraph should be shown.

:::

# Elifs

Multiple `elif` clauses are possible,

::: if false

This first paragraph should NOT be shown.

::: elif false

This second paragraph should NOT be shown.

::: elif true

This third paragraph **should** be shown.

::: elif true

This fourth paragraph should NOT be shown because the above `elif` was.

::: else

This final paragraph should NOT be shown because the above `elif` was.

:::

# Nested

Probably best to be avoided but possible (and not allowed in prop tests),

::: if true

::: if true

This paragraph should be shown.

:::

::: else

This paragraph should not be shown

:::

In the following example the nested if has the `else`,

::: if true

::: if true

This paragraph should be shown.

::: else

This paragraph should not be shown

:::

:::
