Example using Markdown as canonical format to test alternative ways to write instruction blocks.

# Instruction type: new

No closing fence:

::: new

Closing fence at end of line

::: new :::

Closing fence on new line at end of same paragraph

::: new
:::

Closing fence as new paragraph (this would be unusual)

::: new

:::

# Instruction type: edit

No closing fence:

::: edit

Paragraph to edit #1

Closing fence at end of line  (this would be unusual)

::: edit :::

Paragraph to edit #2

Closing fence on new line at end of same paragraph (this would be unusual)

::: edit
:::

Paragraph to edit #3

Closing fence as new paragraph, to bound content

::: edit

Paragraph to edit #4

Paragraph to edit #5

:::