<!-- Generated from site/docs/documents by claude/scripts/gen-references.sh. Do not edit. -->


# Code

Stencila documents can contain code as part of the document narrative, even when that code is not executable. This is useful for tutorials, technical writing, API documentation, method descriptions, and any document that needs to show source code or code fragments.

> [!tip]
> If you want code that executes and produces outputs, see [Execution](execution).

# Inline code

Use backticks for short code fragments inside paragraphs:

```smd
A code fragment: `2 * pi * r^2`.
```

You can also add a language tag when you want to make the language explicit:

```smd
With language: `6 * 7`{python}.
```

# Code blocks

Use fenced code blocks for longer static code listings:

````smd
```python
# Some python code
a = 3
```
````

A code block can also omit the language:

````smd
```
No language
```
````

Code blocks can also be empty or contain only whitespace when needed:

````smd
```

```
````

# Demo code blocks

You can add the `demo` keyword to a fenced code block when you want Stencila to parse the content of the block and render the resulting document content below it.

This is useful in documentation when you want readers to see both the source and the rendered result.

````smd
```smd
> [!note]
> This is rendered from the code block above.
```
````

Unlike `exec`, the `demo` keyword does not execute code. Instead, it treats the code block content as document source, parses it, and renders the parsed result underneath the block.

# When to use static code

Use static code when you want to:

- show example source code
- explain syntax or APIs
- document configuration snippets
- present code without running it

> [!tip]
> If you want the code to run and produce outputs inside the document, see [Execution](execution).

