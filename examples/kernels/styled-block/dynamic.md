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

::: { bg-$color-{{shades.bg}} text-$color-{{shades.txt}} p-$pad }

The dynamically styled paragraph.

:::

A style block attempting to use a non-existent variable. Should show error:

::: { bg-color-$foo }

Not styled.

:::
