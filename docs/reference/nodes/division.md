A division is used to style block content

::: rounded-xl bg-rose-100 p-8 text-2xl text-rose-600 text-center text-justify

This paragraph will have a light rose background with rounded corners, extra padding around it, and large, italicized rose colored text.

:::

::: rounded-xl bg-sky-100 p-4 text(2xl sky-600 center)

This paragraph is styled using a `text(...)` group to reduce the length of the Tailwind expression.

:::

## Using variables in Tailwind expressions

Tailwind expressions can use interpolated

```python exec
color = 'amber'
shade = 50
```

::: bg-$color-$shade text-$color-700 p-4

The color of this paragraph is dependent on the `color` string variable assigned in the preceding code chunk. Try changing the variable to "indigo", or one of the other colors in the Tailwind color palette.

:::

## Using calculated values in Tailwind expressions

```python exec
darkness = 300
```

::: `f"bg-indigo-{darkness} p-4 text-indigo-{min(darkness + 300, 900)}"`{python}

In this paragraph the darkness of both the background and the text is controlled by the `darkness` variable assigned in the preceding code chunk.

:::
