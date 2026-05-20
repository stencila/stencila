---
title: Content Credentials Config
description: Content Credentials configuration.
---

Content Credentials configuration.

Defines workspace-level defaults for signing rendered outputs and
generated assets. Commands and site-specific configuration can override
these values.

Can be a simple boolean, profile shorthand, or detailed configuration, e.g.
```toml
# Enable with the default public profile and automatic signer
content-credentials = true

# Enable with a specific profile
content-credentials = "public"

# Detailed Content Credentials configuration
[content-credentials]
enabled = true
profile = "public"
signer = "auto"
```

**Type:** `ContentCredentialsConfig`

# `enabled`

**Type:** `boolean` (optional)

Whether Content Credentials signing is enabled.

If omitted in the detailed table form, Content Credentials are enabled.

# `profile`

**Type:** `ContentCredentialsProfile` (optional)

The privacy/signing projection profile to use.

Defaults to `public`.

| Value | Description |
|-------|-------------|
| `public` | Public-safe credential metadata. |
| `private` | More local detail for internal sharing. |
| `full` | Full local detail for controlled archives. |

# `signer`

**Type:** `ContentCredentialsSigner` (optional)

The signing backend to use.

Defaults to `auto`.

| Value | Description |
|-------|-------------|
| `auto` | Use Cloud signing when available, otherwise fall back to local signing. |
| `cloud` | Use Stencila Cloud's signing service. |
| `local` | Use the local self-signed signing identity. |


***

This documentation was generated from [`lib.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/lib.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
