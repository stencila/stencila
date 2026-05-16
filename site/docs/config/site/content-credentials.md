---
title: Site Content Credentials Config
description: Content Credentials configuration
---

Content Credentials configuration

Enables C2PA Content Credentials signing for rendered HTML and media.
The web metadata index is only generated for pages that have signed
image assets; pages without signed media do not emit an empty index.

Can be a simple boolean, profile shorthand, or detailed configuration, e.g.
```toml
# Enable with the default public profile
[site]
content-credentials = true

# Enable with a specific profile
[site]
content-credentials = "public"

# Detailed Content Credentials configuration
[site.content-credentials]
profile = "public"
```

**Type:** `SiteContentCredentialsConfig`

# `enabled`

**Type:** `boolean` (optional)

Whether Content Credentials signing is enabled.

If omitted in the detailed table form, Content Credentials are enabled.

# `profile`

**Type:** `SiteContentCredentialsProfile` (optional)

The privacy/signing projection profile to use.

Defaults to `public`.

| Value | Description |
|-------|-------------|
| `public` | Public-safe credential metadata. |
| `private` | More local detail for internal sharing. |
| `full` | Full local detail for controlled archives. |


***

This documentation was generated from [`site.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/site.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
