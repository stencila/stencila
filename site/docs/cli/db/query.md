---
title: "`stencila db query`"
description: Query a workspace database
---

Query a workspace database

# Usage

```sh
stencila db query [OPTIONS] <QUERY> [OUTPUT]
```

# Examples

```bash
# Query the workspace database
stencila db query "workspace.paragraphs()"

# Use Cypher query language
stencila db query --cypher "MATCH (h:Heading) WHERE h.level = 1 RETURN h"
```

# Arguments

| Name       | Description                                   |
| ---------- | --------------------------------------------- |
| `<QUERY>`  | The DocsQL or Cypher query to run.            |
| `[OUTPUT]` | The path of the file to output the result to. |

# Options

| Name           | Description                                                                                         |
| -------------- | --------------------------------------------------------------------------------------------------- |
| `-c, --cypher` | Use Cypher as the query language (instead of DocsQL the default). Possible values: `true`, `false`. |
| `-t, --to`     | The format to output the result as.                                                                 |
| `--compact`    | Use compact form of encoding if possible. Possible values: `true`, `false`.                         |
| `-p, --pretty` | Use a "pretty" form of encoding if possible. Possible values: `true`, `false`.                      |
