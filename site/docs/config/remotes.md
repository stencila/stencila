---
title: Remotes Config
description: Remote synchronization configuration.
---

Remote synchronization configuration.

Maps local paths to remote service URLs. The key is the local path
(file, directory, or pattern), and the value can be:
- A simple URL string: `"site" = "https://example.stencila.site/"`
- An object with watch: `"file.md" = { url = "...", watch = "w123" }`
- Multiple remotes: `"file.md" = [{ url = "...", watch = "..." }, "https://..."]`

Directory paths are implicitly recursive, matching all files within.

```toml
# Remotes for a site and specific files
[remotes]
"site" = "https://example.stencila.site/"
"docs/report.md" = { url = "https://docs.google.com/...", watch = "w123" }
"article.md" = [
  { url = "https://docs.google.com/...", watch = "w456" },
  "https://sharepoint.com/..."
]
```

## RemoteValue Configuration Entry

Value for a remote configuration entry - can be single or multiple targets

Supports both simple cases (one URL) and complex cases (multiple URLs per path).
Each target can be a simple URL string or an object with a watch ID.

### Multiple remote targets for the same path

Use an array when you need to sync a single local file with multiple
remote destinations (e.g., Google Docs and SharePoint), e.g.
```toml
[remotes]
"article.md" = [
  { url = "https://docs.google.com/...", watch = "w456" },
  "https://sharepoint.com/..."
]
```

### Single remote target

The most common form - maps one local path to one remote URL.
Can be a simple URL string or an object with a watch ID, e.g.
```toml
[remotes]
"site" = "https://example.stencila.site/"
"file.md" = { url = "https://...", watch = "w123" }
```


***

This documentation was generated from [`remotes.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/remotes.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
