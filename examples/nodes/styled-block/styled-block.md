A styled block with two paragraphs:

::: {rounded border border-solid border-blue-200 bg-blue-100 p-2}

Styled paragraph one.

Styled paragraph two.

:::

With style language specified:

::: {color:red}

Styled paragraph.

:::

Nested styled blocks:

::: {bg-red-100 p-2}

Outer paragraph.

::::: {bg-blue-100}

Inner paragraph.

:::::

:::

Containing variable interpolations:

::: {bg-red-{{var1}} p-$var2 m-$var3}

Dynamically styled paragraph.

:::