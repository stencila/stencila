---
title: Config
description: Using the Stencila CLI to inspect configuration options
config:
  publish:
    ghost:
      slug: cli-config
      type: post
      state: publish
      tags:
        - '#doc'
        - CLI
---

```sh
stencila config
```

The `stencila config` command can be used to display the configuration parameters of a Stencila file. Configuration paratmeters are used by other parts of the CLI such as `stencila publish` to publish to Zeonodo or Ghost. Config parameters are used to store configuration metadata that are used by these publishers.