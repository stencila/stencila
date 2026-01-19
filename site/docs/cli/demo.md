---
title: "`stencila demo`"
description: Run a terminal demonstration from a document
---

Run a terminal demonstration from a document

# Usage

```sh
stencila demo [OPTIONS] <INPUT> [OUTPUT] [-- <AGG_ARGS>...]
```

# Examples

```bash
# Demo a document in the terminal (uses natural preset by default)
stencila demo document.md

# Record a demo to an animated GIF
stencila demo document.md demo.gif

# Use fast preset for quick, smooth typing
stencila demo document.md --preset fast

# Use fast preset but add some typing variance
stencila demo document.md --preset fast --speed-variance 0.2

# Use fast preset but extend the maximum duration of running times
stencila demo document.md --preset fast --min-running 2000 --max-running 4000

# Use instant preset for immediate results
stencila demo document.md --preset instant

# Disable syntax highlighting for code blocks
stencila demo document.md --no-highlighting

# Demo only specific slides (slides are delimited by ***)
stencila demo document.md --slides 2-4

# Demo multiple slide ranges
stencila demo document.md --slides "1,3-5,7-"
```

# Arguments

| Name         | Description                                              |
| ------------ | -------------------------------------------------------- |
| `<INPUT>`    | The path of the document to demo.                        |
| `[OUTPUT]`   | The path of the recording to generate.                   |
| `[AGG_ARGS]` | Arguments to pass through to `agg` when recoding to GIF. |

# Options

| Name                                          | Description                                                                                                                                                                                                                                                                                                 |
| --------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--preset <PRESET>`                           | Preset for demo style. Possible values: `slow` (Slower typing with some typos and hesitation), `natural` (Average WPM, typo and hesitation rate), `fast` (200 WPM, no hesitation, no typos, consistent code running time), `instant` (Very high WPM and zero code running times). Default value: `natural`. |
| `--speed <SPEED>`                             | Typing speed in words per minute. Default value: `100`.                                                                                                                                                                                                                                                     |
| `--speed-variance <SPEED_VARIANCE>`           | Variance in typing speed (0.0 to 1.0). Default value: `0.3`.                                                                                                                                                                                                                                                |
| `--punctuation-pause <PUNCTUATION_PAUSE>`     | How long to pause after punctuation (milliseconds). Default value: `200`.                                                                                                                                                                                                                                   |
| `--typo-rate <TYPO_RATE>`                     | Probability of making a typo (0.0 to 1.0). Default value: `0`.                                                                                                                                                                                                                                              |
| `--typo-pause <TYPO_PAUSE>`                   | How long to pause after typos before correcting (milliseconds). Default value: `500`.                                                                                                                                                                                                                       |
| `--hesitation-rate <HESITATION_RATE>`         | Probability of brief hesitation (0.0 to 1.0). Default value: `0`.                                                                                                                                                                                                                                           |
| `--hesitation-duration <HESITATION_DURATION>` | Hesitation duration in milliseconds. Default value: `100`.                                                                                                                                                                                                                                                  |
| `--no-highlighting <NO_HIGHLIGHTING>`         | Do not apply syntax highlighting to code. Possible values: `true`, `false`.                                                                                                                                                                                                                                 |
| `--min-running <MIN_RUNNING>`                 | Minimum duration for running spinner in milliseconds. Default value: `500`.                                                                                                                                                                                                                                 |
| `--max-running <MAX_RUNNING>`                 | Maximum duration for running spinner in milliseconds. Default value: `5000`.                                                                                                                                                                                                                                |
| `--slides`                                    | Specify which slides to demo.                                                                                                                                                                                                                                                                               |
| `--no-execute <NO_EXECUTE>`                   | Do not execute the document before running the demo. Possible values: `true`, `false`.                                                                                                                                                                                                                      |
| `--cache <CACHE>`                             | Cache the document after executing it. Possible values: `true`, `false`.                                                                                                                                                                                                                                    |
| `--ignore-errors <IGNORE_ERRORS>`             | Ignore any errors while executing document. Possible values: `true`, `false`.                                                                                                                                                                                                                               |
| `--force-all <FORCE_ALL>`                     | Re-execute all node types regardless of current state. Possible values: `true`, `false`.                                                                                                                                                                                                                    |
| `--skip-code <SKIP_CODE>`                     | Skip executing code. Possible values: `true`, `false`.                                                                                                                                                                                                                                                      |
| `--skip-instructions <SKIP_INSTRUCTIONS>`     | Skip executing instructions. Possible values: `true`, `false`.                                                                                                                                                                                                                                              |
| `--retain-suggestions <RETAIN_SUGGESTIONS>`   | Retain existing suggestions for instructions. Possible values: `true`, `false`.                                                                                                                                                                                                                             |
| `--force-unreviewed <FORCE_UNREVIEWED>`       | Re-execute instructions with suggestions that have not yet been reviewed. Possible values: `true`, `false`.                                                                                                                                                                                                 |
| `--force-accepted <FORCE_ACCEPTED>`           | Re-execute instructions with suggestions that have been accepted. Possible values: `true`, `false`.                                                                                                                                                                                                         |
| `--skip-rejected <SKIP_REJECTED>`             | Skip re-executing instructions with suggestions that have been rejected. Possible values: `true`, `false`.                                                                                                                                                                                                  |
| `--dry-run <DRY_RUN>`                         | Prepare, but do not actually perform, execution tasks. Possible values: `true`, `false`.                                                                                                                                                                                                                    |
