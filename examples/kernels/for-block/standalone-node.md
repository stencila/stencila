::: for item in null {node}

No iterations

:::

::: for item in true

{{ item }}

:::

::: for int in 3

{{ int }}

:::

::: for int in 3.0

{{ int }}

:::

::: for char in "123"

{{ char }}

:::

::: for int in [1, 2, 3]

{{ int }}

:::

::: for pair in `{'a': 1, 'b': 2, 'c': 3}`

{{ `${pair[0]}: ${pair[1]}` }}

:::
