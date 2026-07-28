---
title: "`stencila credentials inspect`"
description: Print the full C2PA manifest data attached to an asset
---

Print the full C2PA manifest data attached to an asset

# Usage

```sh
stencila credentials inspect [OPTIONS] <ASSET>
```

# Arguments

| Name      | Description                   |
| --------- | ----------------------------- |
| `<ASSET>` | Path to the asset to inspect. |

# Options

| Name              | Description                                                                    |
| ----------------- | ------------------------------------------------------------------------------ |
| `-a, --as`        | Output format. Possible values: `json`, `yaml`, `toml`. Default value: `yaml`. |
| `--trust-anchors` | PEM bundle of C2PA trust anchors for local signer trust checks.                |
| `--resources`     | Directory to write binary C2PA resources referenced by the manifest.           |
