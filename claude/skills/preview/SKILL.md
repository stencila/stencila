---
name: preview
description: Open a live preview of a Stencila document or site in the browser.
disable-model-invocation: true
argument-hint: "[path]"
allowed-tools: Bash
---

# /stencila:preview

Open a live preview of the document (or folder) in `$ARGUMENTS`, defaulting
to the current folder.

Use the stencila CLI path reported in session context; otherwise
`${CLAUDE_PLUGIN_OPTION_STENCILA_PATH}` if set, else `stencila`. If no CLI
is available, point the user at `/stencila:doctor` and stop.

```sh
NO_COLOR=1 <stencila> open [path] --yes
```

`open` starts a local preview server and opens the browser; for a folder it
opens the first `index.*`, `main.*`, or `readme.*` file. If the workspace
has a `[site]` configuration and the user wants to preview the whole site,
use `NO_COLOR=1 <stencila> site preview --yes` instead (live reload).

These commands serve until interrupted — run them in the background so the
session is not blocked, and tell the user the local URL from the command
output and that the preview updates as files change.
