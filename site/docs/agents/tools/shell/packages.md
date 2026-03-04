---
title: "Packages"
description: "Guards against destructive package manager operations affecting registries"
---

This page lists the safe and destructive patterns in the **Package Registries** shell guard pack. See [Shell Tool](/docs/agents/tools/shell#guard-pipeline) for how these patterns are evaluated.

## Package Registries

**Pack ID:** `packages.registries`

Guards against destructive package manager operations affecting registries

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `packages.registries.npm_unpublish` | Unpublishing removes a package version from the public registry, potentially breaking dependents | Use `npm deprecate` to mark versions as deprecated instead | High |
| `packages.registries.npm_deprecate` | Deprecating a package version affects all consumers | Verify the package name and version before deprecating | Medium |
| `packages.registries.npm_cache_clean` | Removes the local package cache, requiring full re-download | Use `npm cache verify` to check cache integrity instead | Medium |
| `packages.registries.cargo_publish` | Publishing a crate to crates.io is a public, irreversible action | Verify package metadata with `cargo package --list` first; ensure version and contents are correct | High |
| `packages.registries.npm_publish` | Publishing a package to a registry is a public, irreversible action | Verify package contents with `npm pack --dry-run` first; ensure version and contents are correct | High |

---

This documentation was generated from [`rust/agents/src/tool_guard/shell/packs/packages.rs`](https://github.com/stencila/stencila/blob/main/rust/agents/src/tool_guard/shell/packs/packages.rs).
