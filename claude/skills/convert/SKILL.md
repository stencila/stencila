---
name: convert
description: Convert a document between formats, reporting any conversion losses.
disable-model-invocation: true
argument-hint: <input> <output>
allowed-tools: Bash, Read
---

# /stencila:convert

Convert the input in `$ARGUMENTS` (a path, URL, DOI, arXiv or PubMed
Central id) to the given output path(s).

Use the stencila CLI path reported in session context; otherwise
`${CLAUDE_PLUGIN_OPTION_STENCILA_PATH}` if set, else `stencila`. If no CLI
is available, point the user at `/stencila:doctor` and stop.

```sh
NO_COLOR=1 <stencila> convert <input> <output> --yes
```

Formats are inferred from extensions; add `--from`/`--to` only when they
cannot be. When the target format is lossy (anything other than CBOR, JSON,
JSON5, JSON-LD, YAML and their zipped variants), report what was lost —
losses are printed at `debug` level by default, so add
`--input-losses warn --output-losses warn` to surface them.

For PDFs, `--pages 1,3,5-7` limits which pages are decoded. See the
`conversion` skill for granularity (`--fine`/`--coarse`), structuring
options, and external tools (`--tool pandoc`).

Afterwards, confirm the output written and summarise any reported losses.
