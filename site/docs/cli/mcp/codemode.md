---
title: "`stencila mcp codemode`"
description: Print generated TypeScript declarations for MCP codemode
---

Print generated TypeScript declarations for MCP codemode

Connects to discovered MCP servers, fetches their tools, and prints the `.d.ts` declarations that would be injected at runtime into the codemode sandbox. Useful for verifying what an LLM agent sees.

# Usage

```sh
stencila mcp codemode [OPTIONS]
```

# Examples

```bash
# Print declarations for all available servers
stencila mcp codemode

# Print declarations for a specific server
stencila mcp codemode --server filesystem

# Print declarations for multiple servers
stencila mcp codemode --server filesystem --server github

# Specify a workspace directory
stencila mcp codemode --dir ./my-project
```

# Options

| Name       | Description                                                       |
| ---------- | ----------------------------------------------------------------- |
| `--server` | Only include specific server(s) (repeatable).                     |
| `--dir`    | Workspace directory to discover servers from. Default value: `.`. |
