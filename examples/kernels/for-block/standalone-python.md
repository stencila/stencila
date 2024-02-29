::: for item in None {python}

No iterations

:::

::: for item in True

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

::: for pair in dict(a = 1, b = 2, c = 3)

{{ f"{pair[0]}: {pair[1]}" }}

:::
