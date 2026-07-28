---
title: "`stencila credentials verify`"
description: Verify the C2PA Content Credentials on an asset
---

Verify the C2PA Content Credentials on an asset

# Usage

```sh
stencila credentials verify [OPTIONS] <ASSET>
```

# Arguments

| Name      | Description                  |
| --------- | ---------------------------- |
| `<ASSET>` | Path to the asset to verify. |

# Options

| Name              | Description                                                                              |
| ----------------- | ---------------------------------------------------------------------------------------- |
| `--require`       | Strict requirements; can be passed multiple times.                                       |
| `-a, --as`        | Output format. Defaults to a four-status table. Possible values: `json`, `yaml`, `toml`. |
| `--trust-anchors` | PEM bundle of C2PA trust anchors for local signer trust checks.                          |

**Possible values of `--require`**

| Value                | Description                                                              |
| -------------------- | ------------------------------------------------------------------------ |
| `trusted-signer`     | Require that the signing certificate chains to a trusted anchor          |
| `stencila-assertion` | Require that the manifest carries an `org.stencila.provenance` assertion |
| `repro-exact`        | Require an exact reproducibility match                                   |
