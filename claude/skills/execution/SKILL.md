---
name: execution
description: How Stencila executes documents - kernels, staleness-driven re-execution, execution options, and debugging chunks that error or do not re-run. Use when running stencila execute or render, when a code chunk is not re-running, when choosing a language/kernel, or when reading execution errors.
user-invocable: false
---

# Executing Stencila documents

`stencila execute doc.smd` runs the executable code in a document and saves
the outputs into it. `stencila render doc.smd out.html` is execute **plus**
encode to output formats. Always invoke non-interactively:

```sh
NO_COLOR=1 stencila execute doc.smd --yes
NO_COLOR=1 stencila render doc.smd out.html --yes
```

## Staleness: why a chunk may not re-run

Execution is staleness-driven. A chunk re-runs only when it (or something it
depends on) has changed since its last recorded execution. This is why
"I edited the prose and re-executed, but the plot didn't regenerate" is
expected behaviour, not a bug. Also check the chunk's execution mode in the
source: `lock` prevents re-execution to preserve outputs, `always` forces
re-run every time.

To override staleness:

- `--force-all` — re-execute everything regardless of staleness. The first
  thing to try when outputs look out of date.
- `--skip-code` — skip executing code (useful to re-run only other
  executable node types).
- `--ignore-errors` — do not fail the command on execution errors; render
  what can be rendered.
- `--no-save` (`execute` only) — run without writing outputs back to the
  source document. Use for a reproducibility check that must not mutate
  files.
- `--dry-run` — compile and prepare but do not actually execute.

`execute` and `render` share the execution option group, but `--no-save` is
`execute`-only, and output/encoding options (`--to`, `--theme`, etc.) are
`render`-only. Check `--help` when unsure.

## Kernels

Each language executes in a kernel. `stencila kernels list` shows what is
available on this machine. Current kernels include:

| Kernel | Languages | Notes |
|---|---|---|
| `python` | Python | uses the local environment (e.g. uv, venv) |
| `r` | R | local R installation |
| `nodejs` | JavaScript | local Node.js |
| `quickjs` | JavaScript | builtin, no install needed |
| `bash` | Bash, Shell | |
| `kuzu`, `docsdb`, `docsql` | Cypher, DocsQL | database queries |
| `mermaid`, `graphviz` | Mermaid, DOT | diagrams |
| `jviz` | Cytoscape, ECharts, Plotly, Vega-Lite | JSON visualization specs |
| `jinja` | Jinja | templating |
| `asciimath`, `tex` | AsciiMath, TeX/LaTeX | math |
| `style` | CSS, HTML, Tailwind | styling |

The language on a code chunk selects the kernel (e.g. `python` → the Python
kernel; `js` and `javascript` → a JavaScript kernel). A language with no
matching kernel is a common cause of chunks silently not producing output.

For a quick one-off check that a kernel works, without touching any
document:

```sh
NO_COLOR=1 stencila kernels execute python "1 + 1" --yes
```

## Reading execution errors

Execution messages are attached to the chunk that produced them and printed
to stderr. Work through them in document order — an error in an early chunk
(e.g. an import failure) commonly cascades into "name is not defined" errors
in later chunks; fix the first error first. When a package is missing,
`stencila kernels packages <kernel>` lists what the kernel can see.

## Checking without mutating

For a "does this document still reproduce?" check:

```sh
NO_COLOR=1 stencila execute doc.smd --force-all --no-save --ignore-errors --yes
```

then report the errors, rather than saving over existing outputs.
