---
title: Linting
description: Using the Stencila CLI to lint documents
config:
  publish:
    ghost:
      slug: cli-lint
      type: post
      state: publish
      tags:
        - '#doc'
        - CLI
---

```sh
stencila lint
```
# Overview 

Stencila lint document and check them for formatting errors.

## Usage 

```
stencila lint test.smd
```
Linting messages will be displayed to the user to help fix any linting errors, or a `🎉 No problems found` message will be displayed. 