::: rounded border(& solid blue-200) bg-blue-100 p-2 text-center

A division styled using Tailwind.

:::

::: bg-$color-100 p-2

A division with this parameter &[color]{enum vals=["red","blue","green"]} interpolated into its Tailwind style.

:::

```python exec
# @autorun always
import random
shade = random.choice(range(100, 1000, 100))
```

::: `f"bg-{color}-{shade} p-2"`{python}

A division using the above color with a shade randomly selected in the above Python code chunk.

:::
