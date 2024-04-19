# Stencila Markdown

## Overview

Stencila uses a variant of Github Markdown that includes extensions to support the creation of executable documents.
Adding extensions to Markdown requires choosing something that keeps in the spirit of Markdown, is syntactically distinct for parsing, and is easy to remember and type.
We have chosen a combination of symbols paired with keywords, which we think strikes the right balance.
This section outlines the general approach taken with these extensions.
The following sections outline the specific extensions that are available.

These extensions can be grouped into two types: **block**, **inline**.
"Blocks", in Stencila, are paragraphs, lists, and other elements that are separated by blank lines.
"Inlines" are elements that are part of a block, such as links, images, or emphasis.

The **block** extensions uses the triple colon `:::` to mark out sections or blocks of text, optionally followed by a keyword.
For example, to show the replacement of one block of text for another, we use the following syntax:

```
::: replace

Old paragraph

Another old paragraph

::: with

New paragraph

Another new paragraph

:::
```

Here, the commands `replace` and `with` are used with the `:::` syntax to specify the paragraphs they should modify.
The final `:::` marks the end of the replacement, and thus has no associated command.

The equivalent **inline** extensions use paired double square brackets: `[[` and `]]` with the same `replace` keyword:

```
Some [[replace old>>new]] text.
```

The inline version delimits the replacement with `>>`, and omits the `with` keyword, to make the syntax more concise.
Many of other extensions described below show a similar

Many of the other extensions described below use a similar syntax, having a block version with `:::` and an inline version with `[[` and `]]`.


### Assistants

A key feature of Stencila Markdown is the ability to instruct "assistants" to modify or generate content.
These instructions are prefixed with `do` command and then the `@` symbol to specify the assistant to use.
For example, to modify content in a block, you can use the following syntax:

```
::: do @edit-blocks expand each sentence into a paragraph.
::: with

The fall of the roman empire.

The rise of the byzantine empire.

:::
```

The syntax for this is similar to the `:::` syntax used for blocks, but with the addition of a `do` keyword.

Here is an *inline* example, using a different assistant:

```
The volume of a sphere is given by [[do @insert-math-inline equation for the volume of a sphere]]
```

### Code Blocks and Code Execution

As well as using assistants, Stencila can execute code using *kernels*, such as "R", or "Python", returning data or visualizations.

Code chunks are wrapped in triple backticks, which is a standard Markdown syntax for creating fenced code blocks.
The language of the code block is specified after the opening backticks, such as `python` or `javascript`.
However, code blocks can have an additional `exec` keyword after the language specifier.
This indicates to Stencila that the code block can be executed, using the appropriate kernel.

For example, this code block will be executed:

```python exec
print("Hello, World!")
```

Whereas this code block will not be executed:

```python
# How to count to 10
for i in range(10):
    print(i)
```

### For Loops

The syntax uses ::: for ... ::: to define a for loop block, where you specify the iteration variable and the sequence to iterate over. The content within the block is repeated for each value in the sequence. Nested loops are supported by increasing the number of ::: characters.
An else clause can be added using ::: else ::: to provide alternative content when the sequence is empty. The {exec} tag within the content enables dynamic execution of inline code or variables.

