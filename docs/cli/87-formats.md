---
title: Formats
description: Display formats that can be used in the Stencila CLI
config:
  publish:
    ghost:
      slug: cli-formats
      type: post
      state: publish
      tags:
        - '#doc'
        - CLI
---

```sh
stencila formats
```

# Overview

Stencila formats are types of documents that stencila can convert to and from, each format and its capabilities and limitations are detailed in the Formats section of the documentation. The `stencila formats` command is informational and shows the formats supported in the current Stencila binary.

If you want to convert from / to a particular format, use the [`stencila convert`](docs/cli-convert) command.
