---
title: "`stencila mcp`"
description: Manage MCP (Model Context Protocol) servers
---

Manage MCP (Model Context Protocol) servers

# Usage

```sh
stencila mcp [COMMAND]
```

# Examples

```bash
# List all discovered MCP servers
stencila mcp

# Show details about a specific server
stencila mcp show filesystem

# Show a server's tools (connects to the server)
stencila mcp show filesystem --tools

# Add a stdio server
stencila mcp add filesystem npx -y @modelcontextprotocol/server-filesystem /tmp

# Add an HTTP server
stencila mcp add remote-api https://api.example.com/mcp

# Remove a server
stencila mcp remove filesystem

# Print TypeScript declarations for all servers
stencila mcp codemode

# Print declarations for a specific server only
stencila mcp codemode --server filesystem
```

# Subcommands

| Command                   | Description                                              |
| ------------------------- | -------------------------------------------------------- |
| [`list`](list.md)         | List all discovered MCP servers                          |
| [`show`](show.md)         | Show details of an MCP server                            |
| [`add`](add.md)           | Add an MCP server to stencila.toml                       |
| [`remove`](remove.md)     | Remove an MCP server from stencila.toml                  |
| [`codemode`](codemode.md) | Print generated TypeScript declarations for MCP codemode |
