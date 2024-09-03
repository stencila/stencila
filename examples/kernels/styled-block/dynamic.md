A Bash code chunk that assigns the colour variable:

```bash exec
color=pink
```

A Javascript code chunk that assigns shade variables:

```js exec
var shades = {
  bg: 100,
  txt: 800,
};
```

A Python code chunk that assigns padding variable:

```python exec
pad = 3
```

A paragraph with a dynamic style using Tailwind and the variables interpolated:

::: style bg-$color-{{shades.bg}} text-$color-{{shades.txt}} p-$pad

The dynamically styled paragraph.

:::

A styled block attempting to use a non-existent variable. Should show error:

::: style bg-color-$foo

Not styled.

:::
