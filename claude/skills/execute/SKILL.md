---
name: execute
description: Execute the code in a Stencila document and save the outputs into it.
disable-model-invocation: true
argument-hint: <input>
allowed-tools: Bash, Read, Edit
---

# /stencila:execute

Execute the document given in `$ARGUMENTS`.

Use the stencila CLI path reported in session context; otherwise
`${CLAUDE_PLUGIN_OPTION_STENCILA_PATH}` if set, else `stencila`. If no CLI
is available, point the user at `/stencila:doctor` and stop.

```sh
NO_COLOR=1 <stencila> execute <input> --yes
```

- Execution is staleness-driven: unchanged chunks do not re-run. Add
  `--force-all` when the user wants everything re-executed.
- When the user only wants to *check* the document (not update it), add
  `--no-save` so outputs are not written back.
- `--ignore-errors` continues past failing chunks.

Afterwards, summarise: which chunks ran, which errored (quote the first
error and locate its chunk), and whether the document was saved. See the
`execution` skill for staleness and kernel details.
