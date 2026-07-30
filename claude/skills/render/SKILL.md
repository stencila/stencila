---
name: render
description: Render a Stencila document - execute its code and encode to output formats.
disable-model-invocation: true
argument-hint: <input> [outputs...]
allowed-tools: Bash, Read, Edit
---

# /stencila:render

Render the document given in `$ARGUMENTS` (first argument is the input;
any further arguments are output paths, e.g. `report.html report.pdf`).

Use the stencila CLI path reported in session context; otherwise
`${CLAUDE_PLUGIN_OPTION_STENCILA_PATH}` if set, else `stencila`. If no CLI
is available, point the user at `/stencila:doctor` and stop.

```sh
NO_COLOR=1 <stencila> render <input> [outputs...] --yes
```

- With no output paths, this executes the document and saves it in place.
- Document parameters go after `--`: `... --yes -- --year=2024`.
- Useful flags: `--force-all` (re-execute everything), `--theme <THEME>`,
  `--ignore-errors` (render despite failing chunks).

Afterwards:

1. If execution errors were reported, quote the first error, identify the
   offending chunk in the source, and offer to fix it (see the `execution`
   skill for how errors cascade).
2. If rendering succeeded, confirm which outputs were written.

Report the diagnostic, not the raw wall of output.
