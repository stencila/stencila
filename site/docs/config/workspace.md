---
title: Workspace Config
description: Workspace configuration.
---

Workspace configuration.

**Type:** `WorkspaceConfig`

# `id`

**Type:** `string` (optional)

**Pattern:** `^ws[a-z0-9]{10}$`

The workspace public ID from Stencila Cloud.

A 12-character string: "ws" prefix followed by 10 lowercase alphanumeric
characters (e.g., "ws3x9k2m7fab").

This is automatically assigned when a workspace is created via
`stencila site create` or when pushing to a site for the first time.
The workspace ID is derived from the GitHub repository URL.

# `watch`

**Type:** `string` (optional)

**Pattern:** `^wa[a-z0-9]{10}$`

The workspace watch ID from Stencila Cloud.

A 12-character string: "wa" prefix followed by 10 lowercase alphanumeric
characters (e.g., "wa7x2k9m3fab").

This is set when `stencila watch` is run without a file path to enable
workspace-level watching. When enabled, `update.sh` is run on each git push.


***

This documentation was generated from [`workspace.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/workspace.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
