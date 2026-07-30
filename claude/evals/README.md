# Evals

Behavioural eval cases for the Stencila plugin, run with:

```sh
claude plugin eval ./claude --allow-tools Bash Write Edit
```

These are slow, cost money, and are non-deterministic — they are run
manually before releases, not in CI. Each case directory contains a
`prompt.md` (the user request, run in a scaffold copy of the directory) and
`graders/` (criteria for scoring the outcome). Fixture documents live
alongside the prompt.

| Case | Exercises |
|---|---|
| `add-python-plot` | Authoring: valid `.smd` executable syntax, verification by rendering |
| `convert-docx` | Conversion: DOCX → executable Markdown, loss reporting |
| `fix-stale-chunk` | Execution: staleness diagnosis, `--force-all` |

The `convert-docx` case needs its DOCX fixture generated first:

```sh
cd claude/evals/convert-docx && stencila convert report.smd report.docx --yes
```
