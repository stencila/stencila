---
title: "Datasets"
description: "Guards against destructive operations on scientific data versioning tools. Guards against destructive operations in scientific data transfer tools"
---

This page lists the safe and destructive patterns in the **Data Versioning** and **Data Transfer Tools** shell guard packs. See [Shell Tool](/docs/agents/tools/shell#guard-pipeline) for how these patterns are evaluated.

## Data Versioning

**Pack ID:** `datasets.versioning`

Guards against destructive operations on scientific data versioning tools

### Safe patterns

| Rule ID | Pattern |
|---------|--------|
| `datasets.versioning.dvc_status` | `^dvc\s+status\b[^\|><]*$` |
| `datasets.versioning.dvc_diff` | `^dvc\s+diff\b[^\|><]*$` |
| `datasets.versioning.dvc_params_diff` | `^dvc\s+params\s+diff\b[^\|><]*$` |
| `datasets.versioning.dvc_metrics_show` | `^dvc\s+metrics\s+show\b[^\|><]*$` |
| `datasets.versioning.dvc_plots_show` | `^dvc\s+plots\s+show\b[^\|><]*$` |
| `datasets.versioning.git_annex_whereis` | `^git\s+annex\s+whereis\b[^\|><]*$` |
| `datasets.versioning.git_annex_info` | `^git\s+annex\s+info\b[^\|><]*$` |
| `datasets.versioning.datalad_status` | `^datalad\s+status\b[^\|><]*$` |

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `datasets.versioning.dvc_gc` | Removes cached DVC file versions, potentially losing data not on the current branch | Use `dvc gc --dry` to preview what will be removed; avoid `--all-commits` | High |
| `datasets.versioning.dvc_remove` | Stops tracking files with DVC and removes .dvc metadata | Use `dvc status` to review tracked files first | Medium |
| `datasets.versioning.git_annex_drop_force` | Force-drops annexed file content without verifying remote copies exist | Use `git annex drop` without `--force` to ensure remote copies are verified first | High |
| `datasets.versioning.git_annex_drop` | Drops local copies of annexed file content | Use `git annex whereis` to verify remote copies exist before dropping | Medium |
| `datasets.versioning.datalad_drop` | Drops local file content from a DataLad dataset | Use `datalad status` to review before dropping; avoid `--reckless` | Medium |

## Data Transfer Tools

**Pack ID:** `datasets.transfer`

Guards against destructive operations in scientific data transfer tools

### Safe patterns

| Rule ID | Pattern |
|---------|--------|
| `datasets.transfer.globus_ls` | `^globus\s+ls\b[^\|><]*$` |
| `datasets.transfer.globus_task_list` | `^globus\s+task\s+list\b[^\|><]*$` |
| `datasets.transfer.rclone_ls` | `^rclone\s+(?:ls\|lsd\|lsl\|lsf\|size)\b[^\|><]*$` |
| `datasets.transfer.ils` | `^ils\b[^\|><]*$` |
| `datasets.transfer.iquest` | `^iquest\b[^\|><]*$` |

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `datasets.transfer.globus_delete` | Permanently deletes files on a Globus endpoint | Use `globus ls` to review endpoint contents before deleting | High |
| `datasets.transfer.irods_irm` | Removes files or collections from iRODS | Use `ils` to review the collection before removing | Medium |
| `datasets.transfer.irods_irmtrash` | Permanently purges iRODS trash, making recovery impossible | Review trash contents before purging | High |
| `datasets.transfer.rclone_delete` | Permanently removes files from remote storage | Use `rclone ls` to review files first; prefer `rclone move` with a backup | High |
| `datasets.transfer.rclone_sync_delete` | Syncs source to destination, deleting files at destination that don't exist at source | Use `rclone sync --dry-run` to preview changes; prefer `rclone copy` to avoid deletions | Medium |

---

This documentation was generated from [`rust/agents/src/tool_guard/shell/packs/datasets.rs`](https://github.com/stencila/stencila/blob/main/rust/agents/src/tool_guard/shell/packs/datasets.rs).
