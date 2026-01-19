---
title: "`stencila pull`"
description: Pull a document from a remote service
---

Pull a document from a remote service

# Usage

```sh
stencila pull [OPTIONS] <PATH>
```

# Examples

```bash
# Pull from the tracked remote (if only one exists)
stencila pull document.smd

# Pull from tracked Google Doc
stencila pull document.smd --from gdoc

# Pull from untracked Google Doc
stencila pull document.smd --from https://docs.google.com/document/d/abc123

# Pull from tracked Microsoft 365 document
stencila pull document.smd --from m365

# Pull from GitHub Issue
stencila pull document.smd --from https://github.com/org/repo/issues/123

# Pull without merging (replace local file)
stencila pull document.smd --from gdoc --no-merge

# Pull all documents from email attachments using embedded path metadata
stencila pull - --from https://api.stencila.cloud/v1/watches/wAbC12345/email/attachments
```

# Arguments

| Name     | Description                     |
| -------- | ------------------------------- |
| `<PATH>` | The path to the local document. |

# Options

| Name                    | Description                                                   |
| ----------------------- | ------------------------------------------------------------- |
| `-f, --from`            | The target to pull from.                                      |
| `--no-merge <NO_MERGE>` | Do not merge, just replace. Possible values: `true`, `false`. |
