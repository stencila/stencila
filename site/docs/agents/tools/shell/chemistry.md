---
title: "Chemistry"
description: "Guards against destructive molecular dynamics and chemistry tool operations"
---

This page lists the safe and destructive patterns in the **Molecular Dynamics** shell guard pack. See [Shell Tool](/docs/agents/tools/shell#guard-pipeline) for how these patterns are evaluated.

## Molecular Dynamics

**Pack ID:** `chemistry.molecular_dynamics`

Guards against destructive molecular dynamics and chemistry tool operations

### Safe patterns

| Rule ID | Pattern |
|---------|--------|
| `chemistry.molecular_dynamics.obabel_list` | `^obabel\s+-L\b[^\|><]*$` |
| `chemistry.molecular_dynamics.gmx_check` | `^gmx\s+check\b[^\|><]*$` |
| `chemistry.molecular_dynamics.gmx_dump` | `^gmx\s+dump\b[^\|><]*$` |

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `chemistry.molecular_dynamics.gmx_trjconv_overwrite` | Trajectory conversion can overwrite existing trajectory files | Use `-o` with a new filename; backup the original trajectory first | Medium |
| `chemistry.molecular_dynamics.gmx_eneconv_overwrite` | Energy file conversion can overwrite existing energy files | Use `-o` with a new filename; backup the original file first | Medium |
| `chemistry.molecular_dynamics.obabel_overwrite` | Open Babel format conversion overwrites output files | Verify the output path; backup existing files first | Medium |

---

This documentation was generated from [`rust/agents/src/tool_guard/shell/packs/chemistry.rs`](https://github.com/stencila/stencila/blob/main/rust/agents/src/tool_guard/shell/packs/chemistry.rs).
