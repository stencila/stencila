---
title: "`stencila graph`"
description: Build, view, and export Stencila graphs
---

Build, view, and export Stencila graphs

# Usage

```sh
stencila graph [OPTIONS] [PATH] [OUTPUT]
```

# Examples

```bash
# View the current workspace graph in a browser
stencila graph

# View a workspace graph in a browser
stencila graph .

# Start the graph server without opening a browser
stencila graph . --no-open --port 9010

# Export graph JSON inferred from the output extension
stencila graph . graph.json

# Export graph YAML inferred from the output extension
stencila graph report.smd graph.yaml

# Export graph YAML to stdout
stencila graph . - --to yaml

# Export a projected data flow graph as Graphviz DOT
stencila graph . graph.dot --view flow

# Export a detailed data flow graph including local symbols
stencila graph . graph.dot --view flow --detail high

# Export only the data flow connected to a matching script
stencila graph . graph.png --view flow --connected-to analysis.R

# Export the full connected component through shared inputs
stencila graph . graph.png --view flow --connected-to analysis.R --connected-mode undirected

# Export the same graph without directory/document containment clusters
stencila graph . graph.dot --view flow --containment none

# Export a projected software dependency graph as SVG using Graphviz
stencila graph . graph.svg --view deps

# Report I/O that static analysis could not resolve
stencila graph . --explain
```

# Arguments

| Name       | Description                                                            |
| ---------- | ---------------------------------------------------------------------- |
| `[PATH]`   | The workspace directory or document file to graph. Default value: `.`. |
| `[OUTPUT]` | Output path for exporting the graph, or `-` for stdout.                |

# Options

| Name                      | Description                                                                                                     |
| ------------------------- | --------------------------------------------------------------------------------------------------------------- |
| `--to`                    | Output format, overriding inference from the output extension.                                                  |
| `--view`                  | Projection preset for DOT, SVG, and PNG graph exports. Default value: `auto`.                                   |
| `--detail`                | Detail level for projected graph exports. Default value: `medium`.                                              |
| `--containment`           | How to represent containment in projected graph exports.                                                        |
| `--structure`             | Include structural containment as visual clusters in projected graph exports. Possible values: `true`, `false`. |
| `--no-structure`          | Exclude structural containment context in projected graph exports. Possible values: `true`, `false`.            |
| `--no-low-confidence`     | Exclude low-confidence edges in projected graph exports. Possible values: `true`, `false`.                      |
| `--no-collapse-citations` | Keep citation marker nodes visible in projected graph exports. Possible values: `true`, `false`.                |
| `--explain`               | Report I/O that static analysis could not resolve, instead of serving. Possible values: `true`, `false`.        |
| `--no-c2pa`               | Do not inspect C2PA content credentials while building workspace graphs. Possible values: `true`, `false`.      |
| `--no-git-authors`        | Do not include Git commit authors on file-backed workspace graph nodes. Possible values: `true`, `false`.       |
| `--connected-to`          | Filter projected graph exports to nodes connected to matching nodes.                                            |
| `--connected-mode`        | How to traverse graph edges for connected-to filtering. Default value: `directed`.                              |
| `-a, --address`           | The address to serve on. Default value: `127.0.0.1`.                                                            |
| `-p, --port`              | The port to serve on. Default value: `9000`.                                                                    |
| `--no-open`               | Do not open the graph view in a browser. Possible values: `true`, `false`.                                      |
| `--no-auth`               | Do not authenticate or authorize graph view requests. Possible values: `true`, `false`.                         |

**Possible values of `--view`**

| Value   | Description                                                              |
| ------- | ------------------------------------------------------------------------ |
| `auto`  | Choose the first useful projection from the graph's relationships        |
| `all`   | Show every graph node and edge without applying a focused projection     |
| `flow`  | Show resource flow, data lineage, and provenance relationships           |
| `deps`  | Show software imports, calls, environments, packages, and dependency use |
| `cite`  | Show bibliographic references, citations, and external resource links    |
| `react` | Show executable document reactivity dependencies                         |

**Possible values of `--connected-mode`**

| Value        | Description                                                              |
| ------------ | ------------------------------------------------------------------------ |
| `directed`   | Include upstream dependencies and downstream dependents of matched nodes |
| `undirected` | Include the full undirected component containing matched nodes           |

**Possible values of `--to`**

| Value  | Description                                 |
| ------ | ------------------------------------------- |
| `json` | Stencila Schema Graph as JSON               |
| `yaml` | Stencila Schema Graph as YAML               |
| `dot`  | Projected graph as Graphviz DOT             |
| `svg`  | Projected graph rendered to SVG by Graphviz |
| `png`  | Projected graph rendered to PNG by Graphviz |

**Possible values of `--detail`**

| Value    | Description                                                                    |
| -------- | ------------------------------------------------------------------------------ |
| `low`    | Show only the main resource, code, output, and environment relationships       |
| `medium` | Show useful data-level detail while hiding local symbol and function internals |
| `high`   | Show all relationships selected by the preset                                  |

**Possible values of `--containment`**

| Value      | Description                                                             |
| ---------- | ----------------------------------------------------------------------- |
| `none`     | Do not include structural containment context                           |
| `clusters` | Use containment to group nodes visually, without rendering PartOf edges |
| `edges`    | Render containment as explicit PartOf edges                             |
| `both`     | Use both visual groups and explicit PartOf edges                        |
