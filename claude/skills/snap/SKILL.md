---
name: snap
description: Screenshot a Stencila document and read it back for visual verification.
disable-model-invocation: true
argument-hint: <path> [selector]
allowed-tools: Bash, Read
---

# /stencila:snap

Capture a screenshot of the document in `$ARGUMENTS` (optionally cropped to
a CSS selector given as the second argument) and inspect it.

Use the stencila CLI path reported in session context; otherwise
`${CLAUDE_PLUGIN_OPTION_STENCILA_PATH}` if set, else `stencila`. If no CLI
is available, point the user at `/stencila:doctor` and stop.

```sh
NO_COLOR=1 <stencila> snap <path> --shot /tmp/stencila-snap.png --yes
```

- Add `--selector "<selector>"` to crop to an element.
- `--full` captures the whole scrollable page rather than the first screen.
- `--device mobile|tablet|laptop|desktop` and `--dark`/`--light` vary the
  viewport and color scheme when checking responsive or themed output.

Then **Read the PNG** and describe what you actually see — layout problems,
missing figures, unstyled elements — rather than assuming the render is
fine. This is the visual half of verification; execution errors are the
semantic half (see the `execution` skill).

Screenshotting requires a Chrome/Chromium; if `snap` fails on that, suggest
`/stencila:doctor` to check external tools.
