---
title: "`stencila new`"
description: Create a new, tracked, document
---

Create a new, tracked, document

# Usage

```sh
stencila new [OPTIONS] <PATH>
```

# Examples

```bash
# Create a new article (default)
stencila new my-article.md

# Create a new chat document
stencila new conversation.md --type chat

# Create a new AI prompt
stencila new template.md --type prompt

# Create a document in a subdirectory
stencila new docs/report.md

# Overwrite an existing document
stencila new existing.md --force
```

# Arguments

| Name     | Description                         |
| -------- | ----------------------------------- |
| `<PATH>` | The path of the document to create. |

# Options

| Name                  | Description                                                                                             |
| --------------------- | ------------------------------------------------------------------------------------------------------- |
| `-f, --force <FORCE>` | Overwrite the document, if it already exists. Possible values: `true`, `false`.                         |
| `-t, --type <TYPE>`   | The type of document to create. Possible values: `article`, `chat`, `prompt`. Default value: `article`. |
