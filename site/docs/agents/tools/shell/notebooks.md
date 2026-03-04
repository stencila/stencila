---
title: "Notebooks"
description: "Guards against destructive Jupyter notebook operations"
---

This page lists the safe and destructive patterns in the **Jupyter Notebooks** shell guard pack. See [Shell Tool](/docs/agents/tools/shell#guard-pipeline) for how these patterns are evaluated.

## Jupyter Notebooks

**Pack ID:** `notebooks.jupyter`

Guards against destructive Jupyter notebook operations

### Safe patterns

| Rule ID | Pattern |
|---------|--------|
| `notebooks.jupyter.jupyter_kernelspec_list` | `^jupyter\s+kernelspec\s+list\b[^\|><]*$` |

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `notebooks.jupyter.nbstripout` | Strips output cells from notebooks, permanently removing computed results | Backup the notebook first; use `nbstripout --dry-run` if available to preview | Medium |
| `notebooks.jupyter.nbconvert_inplace` | In-place notebook conversion overwrites the original file | Use `--output` to write to a new file instead of `--inplace` | Medium |
| `notebooks.jupyter.jupyter_kernelspec_uninstall` | Removes a Jupyter kernel specification | Use `jupyter kernelspec list` to review installed kernels first | Medium |

---

This documentation was generated from [`rust/agents/src/tool_guard/shell/packs/notebooks.rs`](https://github.com/stencila/stencila/blob/main/rust/agents/src/tool_guard/shell/packs/notebooks.rs).
