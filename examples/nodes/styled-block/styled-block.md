A styled block with two paragraphs:

::: style rounded border border-solid border-blue-200 bg-blue-100 p-2

Styled paragraph one.

Styled paragraph two.

:::

With style language specified:

::: style color:red

Styled paragraph.

:::

Nested styled blocks:

::: style bg-red-100 p-2

Outer paragraph.

::::: style bg-blue-100

Inner paragraph.

:::::

:::

Containing variable interpolations:

::: style bg-red-{{var1}} p-$var2 m-$var3

Dynamically styled paragraph.

:::
