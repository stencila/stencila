---
title: Site Content Credentials Config
description: Site Content Credentials configuration
---

Site Content Credentials configuration

Overrides workspace-level C2PA Content Credentials defaults for rendered
HTML and media.
The web metadata index is only generated for pages that have signed
image assets; pages without signed media do not emit an empty index.

Can be a simple boolean, profile shorthand, or detailed configuration, e.g.
```toml
# Enable using workspace defaults
[site]
content-credentials = true

# Enable and override the profile
[site]
content-credentials = "public"

# Detailed site Content Credentials configuration
[site.content-credentials]
enabled = true
profile = "public"
signer = "cloud"
soft-binding = true
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

# `soft-binding`

**Type:** `boolean` (optional)

Whether Stencila Cloud should store the manifest and register a soft
binding for each signed asset.

Defaults to `true`. Local signing ignores this and emits a warning.


***

This documentation was generated from [`site.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/site.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
