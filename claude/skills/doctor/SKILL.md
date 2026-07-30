---
name: doctor
description: Check the health of the Stencila CLI installation - version, available upgrades, kernels, linters, external tools, and workspace configuration. The first thing to run when anything Stencila-related is not working.
disable-model-invocation: true
argument-hint: ""
allowed-tools: Bash
---

# /stencila:doctor

Diagnose the Stencila installation and workspace. Report findings as a short
summary, not raw command output.

## 1. Resolve the CLI

Use the first of these that exists (the same ladder the plugin's
SessionStart hook uses — if session context already reports a resolved path,
use that and skip to step 2):

1. The path in the plugin's `stencila_path` setting:
   `${CLAUDE_PLUGIN_OPTION_STENCILA_PATH}` (if set and not the default
   `stencila`)
2. `./target/debug/stencila` (when working in the Stencila source repo)
3. `stencila` on PATH (`command -v stencila`)
4. `/usr/local/bin/stencila`, `~/.local/bin/stencila`, `~/.cargo/bin/stencila`

**If no CLI is found**: report that Stencila is not installed and give the
platform-appropriate install command —

- macOS / Linux: `curl -LsSf https://stencila.io/install.sh | bash`
- Windows: download the ZIP for the latest release from
  <https://github.com/stencila/stencila/releases>

Offer to run the installer, but **never run it unprompted** — wait for the
user to say yes. Then stop; the remaining checks need a CLI.

## 2. Run the checks

With `<stencila>` as the resolved path, run (all with `NO_COLOR=1` and
`--yes`):

```sh
NO_COLOR=1 <stencila> --version --yes           # version (plugin tested against 2.14)
NO_COLOR=1 <stencila> upgrade --check --yes     # available upgrade (does NOT install)
NO_COLOR=1 <stencila> kernels list --yes        # execution kernels and availability
NO_COLOR=1 <stencila> linters list --yes        # linters
NO_COLOR=1 <stencila> tools list --yes          # external tools (pandoc, chrome, etc.)
```

Also check the workspace: does `stencila.toml` (or `stencila.local.toml`)
exist in this or an ancestor directory? If yes, run
`NO_COLOR=1 <stencila> status --no-remotes --yes` for tracked-document
status.

## 3. Report

Summarise in a few lines:

- Resolved CLI path and version; warn (do not fail) if the version is below
  2.14, and mention `stencila upgrade` if an upgrade is available — do not
  run it without being asked.
- Kernels relevant to the user's documents that are unavailable (e.g. R
  documents but no R kernel).
- Whether a workspace is configured, and any tracked documents needing
  attention.
- Anything that looks broken, with the exact command that revealed it.
