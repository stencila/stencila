---
title: "`stencila query`"
description: Query a workspace database
---

Query a workspace database

# Usage

```sh
stencila query [OPTIONS] <FILE> <QUERY> [OUTPUT]
```

# Examples

```bash
# Query a specific document
stencila query article.qmd "paragraphs().sample(3)"

# Query with output to file
stencila query report.myst "headings(.level == 1)" headings.md

# Use Cypher query language
stencila query doc.ipynb --cypher "MATCH (h:Heading) WHERE h.level = 1 RETURN h"
```

# Arguments

| Name       | Description                                   |
| ---------- | --------------------------------------------- |
| `<FILE>`   | The document to query.                        |
| `<QUERY>`  | The DocsQL or Cypher query to run.            |
| `[OUTPUT]` | The path of the file to output the result to. |

# Options

| Name                        | Description                                                                                         |
| --------------------------- | --------------------------------------------------------------------------------------------------- |
| `-c, --cypher <CYPHER>`     | Use Cypher as the query language (instead of DocsQL the default). Possible values: `true`, `false`. |
| `--no-compile <NO_COMPILE>` | Do not compile the document before querying it. Possible values: `true`, `false`.                   |
| `-t, --to`                  | The format to output the result as.                                                                 |
| `--compact <COMPACT>`       | Use compact form of encoding if possible. Possible values: `true`, `false`.                         |
| `-p, --pretty <PRETTY>`     | Use a "pretty" form of encoding if possible. Possible values: `true`, `false`.                      |
