---
title: "Workflows"
description: "Guards against destructive operations in scientific workflow engines"
---

This page lists the safe and destructive patterns in the **Workflow Engines** shell guard pack. See [Shell Tool](/docs/agents/tools/shell#guard-pipeline) for how these patterns are evaluated.

## Workflow Engines

**Pack ID:** `workflows.engines`

Guards against destructive operations in scientific workflow engines

### Safe patterns

| Rule ID | Pattern |
|---------|--------|
| `workflows.engines.nextflow_log` | `^nextflow\s+log\b[^\|><]*$` |
| `workflows.engines.nextflow_list` | `^nextflow\s+list\b[^\|><]*$` |
| `workflows.engines.snakemake_summary` | `^snakemake\s+--summary\b[^\|><]*$` |
| `workflows.engines.snakemake_dryrun` | `^snakemake\s+(?:--dryrun\|-n\|--dry-run)\b[^\|><]*$` |

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `workflows.engines.nextflow_clean` | Removes cached workflow data and intermediate results | Use `nextflow log` to review runs before cleaning | Medium |
| `workflows.engines.nextflow_drop` | Removes a cached pipeline from local storage | Use `nextflow list` to review cached pipelines first | Medium |
| `workflows.engines.snakemake_delete_all` | Deletes all output files produced by the workflow | Review outputs with `snakemake --summary` first | High |
| `workflows.engines.snakemake_unlock` | Force-unlocks a workflow directory, risking concurrent execution conflicts | Ensure no other Snakemake instance is running first | Medium |

---

This documentation was generated from [`rust/agents/src/tool_guard/shell/packs/workflows.rs`](https://github.com/stencila/stencila/blob/main/rust/agents/src/tool_guard/shell/packs/workflows.rs).
