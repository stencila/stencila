---
title: "`stencila credentials`"
description: Manage and inspect Stencila C2PA Content Credentials
---

Manage and inspect Stencila C2PA Content Credentials

# Usage

```sh
stencila credentials <COMMAND>
```

# Examples

```bash
# Generate a local signing identity (untrusted)
stencila credentials init

# Sign an existing image with a C2PA manifest
stencila credentials sign figure.png

# Sign with an explicit title
stencila credentials sign figure.png --title "Figure 4"

# Verify an asset and show the four-status report
stencila credentials verify figure.png

# Verify and require a Stencila assertion to be present
stencila credentials verify figure.png --require stencila-assertion

# Verify and emit the full report as JSON
stencila credentials verify figure.png --as json

# Inspect the full manifest as JSON
stencila credentials inspect figure.png

# Refresh the official C2PA trust list cache
stencila credentials trust refresh

Trust
Identities created by credentials init are local self-signed
certificates. They are not on any C2PA trust list and will display as
*untrusted* in third-party verifiers (Adobe, contentcredentials.org).
Use them for local and internal workflows only.
```

# Subcommands

| Command                   | Description                                                                         |
| ------------------------- | ----------------------------------------------------------------------------------- |
| [`init`](init.md)         | Generate a local self-signed signing identity                                       |
| [`sign`](sign.md)         | Sign an asset with a C2PA manifest carrying the `org.stencila.provenance` assertion |
| [`verify`](verify.md)     | Verify the C2PA Content Credentials on an asset                                     |
| [`inspect`](inspect.md)   | Print the full C2PA manifest data attached to an asset                              |
| [`trust`](trust/index.md) | Manage the cached official C2PA trust list                                          |
