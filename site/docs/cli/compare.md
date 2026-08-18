---
title: "`stencila compare`"
description: Compare two documents
---

Compare two documents

Compares the two documents semantically, rather than as text, and reports how their nodes correspond and differ. Neither document is presumed correct: they are simply the left and right snapshots that you selected.

Exits with 0 when the documents are equal, 1 when they differ, and 2 on error.

# Usage

```sh
stencila compare [OPTIONS] <LEFT> <RIGHT> [OUTPUT]
```

# Examples

```bash
# Compare two documents in the terminal
stencila compare before.smd after.smd

# Compare documents in different formats
stencila compare original.smd roundtripped.docx

# Only report how many differences there are
stencila compare before.smd after.smd --summary

# Open a side-by-side view of the comparison in a browser
stencila compare before.smd after.smd --view

# Write the side-by-side view to a HTML file, without opening it
stencila compare before.smd after.smd comparison.html

# Write the comparison artifact as JSON to stdout
stencila compare before.smd after.smd --to json

# Write the comparison artifact to a file
stencila compare before.smd after.smd comparison.yaml

# Ignore differences that are only about identifiers
stencila compare before.smd after.jats.xml --exclude id

# Ignore identifiers everywhere except on figures
stencila compare before.smd after.jats.xml --exclude id --include Figure.id

# Ignore everything about links, and JATS reference types anywhere
stencila compare before.smd after.jats.xml --exclude Link --exclude jatsRefType

# Report nothing but heading differences
stencila compare before.smd after.smd --exclude all --include Heading

# Override the format of an input document
stencila compare before.txt after.smd --left-from smd
```

# Arguments

| Name       | Description                     |
| ---------- | ------------------------------- |
| `<LEFT>`   | The path of the left document.  |
| `<RIGHT>`  | The path of the right document. |
| `[OUTPUT]` | The path of the output file.    |

# Options

| Name                      | Description                                                                                |
| ------------------------- | ------------------------------------------------------------------------------------------ |
| `--left-from`             | The format of the left document.                                                           |
| `--right-from`            | The format of the right document.                                                          |
| `-t, --to`                | The format of the output.                                                                  |
| `--view`                  | Open a side-by-side view of the comparison in a browser. Possible values: `true`, `false`. |
| `--summary`               | Only report counts, not individual differences. Possible values: `true`, `false`.          |
| `--input-losses`          | Action when there are losses decoding either input document. Default value: `warn`.        |
| `--include`               | Only report differences matching this selector.                                            |
| `--exclude`               | Do not report differences matching this selector.                                          |
| `--alignment-cell-budget` | The maximum number of candidate cells that sequence alignment may use.                     |

**Possible values of `--to`**

| Value  | Description                                         |
| ------ | --------------------------------------------------- |
| `text` | A human-readable report                             |
| `html` | A side-by-side report as a self-contained HTML page |
| `json` | The comparison artifact as JSON                     |
| `yaml` | The comparison artifact as YAML                     |

**Possible values of `--input-losses`**

| Value    | Description                    |
| -------- | ------------------------------ |
| `warn`   | Warn about losses and continue |
| `ignore` | Say nothing about losses       |
| `abort`  | Abort without comparing        |
