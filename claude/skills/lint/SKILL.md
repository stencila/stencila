---
name: lint
description: Lint Stencila documents and summarise the diagnostics.
disable-model-invocation: true
argument-hint: "[files...]"
allowed-tools: Bash, Read, Edit
---

# /stencila:lint

Lint the files given in `$ARGUMENTS` (default: all Stencila documents —
`.smd`, `.myst`, `.qmd` — in the project).

Use the stencila CLI path reported in session context; otherwise
`${CLAUDE_PLUGIN_OPTION_STENCILA_PATH}` if set, else `stencila`. If no CLI
is available, point the user at `/stencila:doctor` and stop.

```sh
NO_COLOR=1 <stencila> lint <files...> --as json --yes
```

Parse the JSON diagnostics and summarise them grouped by file: count by
severity, then the individual messages with locations. Do not dump raw
JSON.

Only if the user asks to fix issues, re-run with `--fix` (rewrites files),
or `--format` for formatting-only fixes, then report what changed.
