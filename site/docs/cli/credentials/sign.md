---
title: "`stencila credentials sign`"
description: Sign an asset with a C2PA manifest carrying the `org.stencila.provenance` assertion
---

Sign an asset with a C2PA manifest carrying the `org.stencila.provenance` assertion.

For PNG, JPEG, WebP, SVG, and PDF the manifest is embedded directly in the asset. For other formats the manifest is written to a `.c2pa` sidecar file next to the asset.

# Usage

```sh
stencila credentials sign [OPTIONS] <INPUT>
```

# Arguments

| Name      | Description                |
| --------- | -------------------------- |
| `<INPUT>` | Path to the asset to sign. |

# Options

| Name           | Description                                                                                   |
| -------------- | --------------------------------------------------------------------------------------------- |
| `-o, --output` | Where to write the signed asset (defaults to in-place).                                       |
| `--cert`       | Path to the signing certificate (PEM).                                                        |
| `--key`        | Path to the signing private key (PEM).                                                        |
| `--tsa-url`    | Timestamp authority URL to use when signing.                                                  |
| `--title`      | Title to record in the manifest. Defaults to the asset filename.                              |
| `-a, --as`     | Output format. Defaults to a human-readable summary. Possible values: `json`, `yaml`, `toml`. |
