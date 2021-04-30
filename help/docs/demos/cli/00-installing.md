---
sidebar_label: Installing
title: Install the Stencila CLI
keywords:
  - install
  - Stencila
  - CLI
description: |
  See how to quickly install the Stencila CLI on MacOS or Linux.
---

import AsciinemaPlayer from '../../../src/components/asciinema/player'
import installingDemo from './00-installing.cast'

:::info
This quick little demo shows you how to install the Stencila CLI using the install script if you are on MacOS or Linux. For Windows, please see [these instructions](https://github.com/stencila/stencila/tree/master/cli#-install).
:::

## Demo

<AsciinemaPlayer src={installingDemo} />

## Script

:::tip Run this demo
The above recording was automatically generated from the following Markdown. Copy the code blocks to run the demo yourself.
:::

The easiest way to install the Stencila CLI on MacOS or Linux is to run the install script:

```bash
curl -L https://raw.githubusercontent.com/stencila/stencila/master/install.sh | bash
```

If you are on Windows, or would prefer to install manually, see https://github.com/stencila/stencila/tree/master/cli#-install.

To check that the CLI was installed:

```bash pause=2
stencila --version
```

Success! Now you have the CLI installed, why not check out the other demos on what you can use it for.
