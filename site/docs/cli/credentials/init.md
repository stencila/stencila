---
title: "`stencila credentials init`"
description: Generate a local self-signed signing identity
---

Generate a local self-signed signing identity.

Creates `local-signing-cert.pem` and `local-signing-key.pem` under `<config>/credentials/`. The certificate is **not** trusted by third-party verifiers; use it for local and internal workflows only.

# Usage

```sh
stencila credentials init [OPTIONS]
```

# Options

| Name      | Description                                                                     |
| --------- | ------------------------------------------------------------------------------- |
| `--force` | Overwrite an existing local signing identity. Possible values: `true`, `false`. |
