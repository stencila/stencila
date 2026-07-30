---
name: authoring
description: How to write Stencila Markdown (.smd) and work with executable documents in related flavours (.myst, .qmd). Use when creating or editing .smd files, adding executable code chunks, inline expressions, parameters, figures, tables, math, or document metadata, or when unsure how Stencila Markdown differs from plain Markdown.
user-invocable: false
---

# Authoring Stencila documents

Stencila Markdown (`.smd`) is CommonMark plus a small set of extensions for
executable, reproducible documents. Stencila also parses MyST (`.myst`) and
Quarto (`.qmd`) Markdown; prefer `.smd` for new documents unless the user
already works in one of those ecosystems, in which case keep their format.
The Markdown parser auto-detects the flavour, so `--from` is rarely needed.

Minimum CLI version: 2.14. Verify a document parses with:

```sh
NO_COLOR=1 stencila convert doc.smd --to json --yes > /dev/null
```

## What differs from CommonMark

The constructs below are the ones worth double-checking; everything else
(headings, emphasis, links, lists, block quotes) is standard Markdown.

### Executable code: `exec`

A fenced code block with the `exec` keyword executes when the document is
rendered, and its outputs become part of the document:

````smd
```python exec
import pandas as pd
data = pd.read_csv("data.csv")
data.describe()
```
````

Without `exec` a code block is static, display-only code. Do not confuse
`exec` with `demo`: `demo` parses-and-renders the block content as document
source without executing anything — it is for documentation that shows
source alongside rendered output.

Execution modes go after `exec` (e.g. `always` to re-run on every execution,
`lock` to preserve existing outputs):

````smd
```r exec always
plot(data)
```
````

### Inline expressions

Backtick code with a language tag and `exec` in braces embeds a computed
value in prose:

```smd
The answer is `6 * 7`{python exec}.
```

Without `exec` (e.g. `` `2 * pi`{python} ``) it is just static inline code
with a language.

### Parameters

Named inputs that can be supplied at render time with
`stencila render doc.smd out.html -- --name=value`:

```smd
Analysis for &[year]{int def=2024} using threshold &[cutoff]{num def=0.05}.
```

### Frontmatter

YAML frontmatter carries metadata (title, authors, description):

```smd
---
title: Analysis of penguin morphology
description: Bill dimensions across three species.
---

# Introduction
```

### Figures, tables, and other colon-fenced blocks

Captioned, labelled figures and tables wrap content in `::: figure` /
`::: table` blocks; code chunks inside them produce the figure or table
content:

````smd
::: figure 1

```r exec
plot(y ~ x)
```

Y against X.

:::
````

## Reference files

Read the relevant reference before writing a construct you are not sure
about — they are generated from Stencila's own documentation and are
authoritative:

| Topic | Reference |
|---|---|
| Document structure, headings, paragraphs | `references/basics.md` |
| Static code blocks and inline code | `references/code.md` |
| Executable chunks, expressions, modes, parameters | `references/execution.md` |
| Including other documents, calling with arguments | `references/include-call.md` |
| Math (TeX and AsciiMath, block and inline) | `references/math.md` |
| Figures, captions, labels | `references/figures.md` |
| Tables, captioned tables | `references/tables.md` |
| Lists, checklists | `references/lists.md` |
| Admonitions / callouts | `references/admonitions.md` |
| Sections and semantic structure | `references/sections.md` |
| Frontmatter metadata | `references/metadata.md` |
| Images, audio, video | `references/media.md` |
| Footnotes | `references/notes.md` |

## Verifying your work

After writing or editing a Stencila document:

1. Lint it: `NO_COLOR=1 stencila lint doc.smd --as json --yes`
2. If it has executable code, render it and check for execution errors
   (see the `execution` skill for how execution and staleness work).

Never report an executable document as finished without executing it.
