---
name: new
description: Create a new, tracked Stencila document with scaffolded frontmatter.
disable-model-invocation: true
argument-hint: <path>
allowed-tools: Bash, Read, Edit
---

# /stencila:new

Create a new document at the path in `$ARGUMENTS`.

Use the stencila CLI path reported in session context; otherwise
`${CLAUDE_PLUGIN_OPTION_STENCILA_PATH}` if set, else `stencila`. If no CLI
is available, point the user at `/stencila:doctor` and stop.

```sh
NO_COLOR=1 <stencila> new <path> --yes
```

This creates a tracked article (`--type article` is the default; do not use
`--type chat` unless explicitly asked). Prefer an `.smd` extension unless
the user works in MyST or Quarto.

Then scaffold frontmatter appropriate to what the user is writing — at
minimum a `title` and `description`:

```smd
---
title: <title>
description: <one-sentence description>
---
```

Ask what the document is about if it is not clear from context. See the
`authoring` skill before adding executable content.
