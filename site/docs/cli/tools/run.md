---
title: "`stencila tools run`"
description: Run a command with automatic environment detection and setup
---

Run a command with automatic environment detection and setup

Mainly for testing configurations. Executes a command within the appropriate development environment by automatically detecting and configuring environment managers. This ensures commands run with the correct tool versions and dependencies as specified in the project configuration.

The command automatically detects and chains environment managers: (1) Environment managers (e.g devbox, mise, pixi) - for tool version management (2) Package managers (e.g uv) - for language-specific dependencies

# Usage

```sh
stencila tools run [OPTIONS] [COMMAND]...
```

# Examples

```bash
# Run Python script with automatic environment detection
stencila tools run -- python script.py

# Run Python code
stencila tools run -- python -c "print('hello')"

# Run from a different directory
stencila tools run -C /path/to/project -- npm test

# Run a complex command with multiple arguments
stencila tools run -- pandoc input.md -o output.pdf --pdf-engine=xelatex
```

# Arguments

| Name        | Description                                          |
| ----------- | ---------------------------------------------------- |
| `[COMMAND]` | The command and arguments to run (specify after --). |

# Options

| Name        | Description                        |
| ----------- | ---------------------------------- |
| `-C, --cwd` | Working directory for the command. |

# Note

Use '--' to separate the run command options from the command to execute.
This prevents argument parsing conflicts
