---
title: "Scientific"
description: "Guards against destructive operations in scientific computing environments"
---

This page lists the safe and destructive patterns in the **Scientific Computing** shell guard pack. See [Shell Tool](/docs/agents/tools/shell#guard-pipeline) for how these patterns are evaluated.

## Scientific Computing

**Pack ID:** `scientific.computing`

Guards against destructive operations in scientific computing environments

### Safe patterns

| Rule ID | Pattern |
|---------|--------|
| `scientific.computing.julia_version` | `^julia\s+--version\b[^\|><]*$` |
| `scientific.computing.matlab_ver` | `^matlab\b.*\bver\b[^\|><]*$` |

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `scientific.computing.julia_pkg_gc` | Removes unused Julia package versions from the package depot | Use `Pkg.status()` to review installed packages first | Medium |
| `scientific.computing.julia_pkg_rm` | Removes Julia packages, potentially breaking dependent packages | Use `Pkg.status()` to review dependencies before removing | Medium |
| `scientific.computing.matlab_delete` | Deletes files from within a MATLAB session | Verify file paths with `dir()` before deleting | Medium |
| `scientific.computing.matlab_rmdir` | Removes directories from within a MATLAB session | Verify directory contents with `dir()` before removing | Medium |
| `scientific.computing.octave_unlink` | Deletes files from within an Octave session | Verify file paths before deleting | Medium |

---

This documentation was generated from [`rust/agents/src/tool_guard/shell/packs/scientific.rs`](https://github.com/stencila/stencila/blob/main/rust/agents/src/tool_guard/shell/packs/scientific.rs).
