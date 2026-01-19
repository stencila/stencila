---
title: "`stencila logs`"
description: Display logs from Stencila Cloud workspace sessions
---

Display logs from Stencila Cloud workspace sessions

# Usage

```sh
stencila logs [OPTIONS] --session <SESSION>
```

# Examples

```bash
# View logs for a session
stencila cloud logs --session SESSION_ID

# View last 50 logs
stencila cloud logs --session SESSION_ID --limit 50

# Follow logs (poll every 5 seconds by default)
stencila cloud logs --session SESSION_ID --follow

# Follow logs with custom polling interval
stencila cloud logs --session SESSION_ID --follow 10

# Filter logs by level
stencila cloud logs --session SESSION_ID --level error
```

# Options

| Name            | Description                                                            |
| --------------- | ---------------------------------------------------------------------- |
| `-s, --session` | The session ID to retrieve logs for.                                   |
| `-l, --limit`   | Maximum number of recent logs to display.                              |
| `-f, --follow`  | Continuously poll for new logs every N seconds (press Ctrl+C to stop). |
| `--level`       | Filter logs by level (error, warn, info, debug, trace).                |
