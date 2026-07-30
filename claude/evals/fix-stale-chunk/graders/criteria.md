# Grading criteria

Score the transcript and the final state of `trend.smd` against:

1. **Correct diagnosis.** The agent identified the `lock` execution mode on
   the figure chunk as the reason it does not re-run (and/or explained
   staleness-driven execution). Guessing at unrelated causes (kernel
   problems, matplotlib bugs) without checking the source is a failure.
2. **Correct fix.** The `lock` keyword was removed (or changed to an
   appropriate mode), after which the chunk re-runs.
3. **Verification.** The agent re-executed with the stencila CLI
   (`NO_COLOR=1 ... --yes`, using `--force-all` where appropriate) and
   confirmed the figure chunk ran without errors before reporting done.
4. **The document still parses**:
   `stencila convert trend.smd --to json --yes` succeeds.
