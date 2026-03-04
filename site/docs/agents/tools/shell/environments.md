---
title: "Environments"
description: "Guards against destructive operations on environment and package managers. Guards against destructive R package management operations"
---

This page lists the safe and destructive patterns in the **Environment Managers** and **R Language** shell guard packs. See [Shell Tool](/docs/agents/tools/shell#guard-pipeline) for how these patterns are evaluated.

## Environment Managers

**Pack ID:** `environments.managers`

Guards against destructive operations on environment and package managers

### Safe patterns

| Rule ID | Pattern |
|---------|--------|
| `environments.managers.python_version` | `^python[23]?\s+--version\b[^\|><]*$` |
| `environments.managers.pip_list` | `^pip[3]?\s+list\b[^\|><]*$` |
| `environments.managers.pip_show` | `^pip[3]?\s+show\b[^\|><]*$` |
| `environments.managers.pip_freeze` | `^pip[3]?\s+freeze\b[^\|><]*$` |
| `environments.managers.conda_list` | `^conda\s+list\b[^\|><]*$` |
| `environments.managers.conda_env_list` | `^conda\s+env\s+list\b[^\|><]*$` |
| `environments.managers.conda_env_export` | `^conda\s+env\s+export\b[^\|><]*$` |
| `environments.managers.conda_info` | `^conda\s+info\b[^\|><]*$` |
| `environments.managers.mamba_list` | `^mamba\s+list\b[^\|><]*$` |
| `environments.managers.mamba_env_list` | `^mamba\s+env\s+list\b[^\|><]*$` |
| `environments.managers.uv_pip_list` | `^uv\s+pip\s+list\b[^\|><]*$` |
| `environments.managers.uv_pip_show` | `^uv\s+pip\s+show\b[^\|><]*$` |
| `environments.managers.spack_find` | `^spack\s+find\b[^\|><]*$` |
| `environments.managers.spack_info` | `^spack\s+info\b[^\|><]*$` |
| `environments.managers.spack_list` | `^spack\s+list\b[^\|><]*$` |

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `environments.managers.conda_env_remove` | Permanently removes a Conda environment and all installed packages | Use `conda env export` to save the environment specification first | High |
| `environments.managers.conda_clean_all` | Removes all cached packages, tarballs, and index caches | Use `conda clean --dry-run --all` to preview what will be removed | Medium |
| `environments.managers.pip_uninstall` | Uninstalls Python packages which may break dependent packages | Use `pip show <package>` to review dependents before uninstalling | Medium |
| `environments.managers.virtualenv_remove` | Removes a Python virtual environment directory | Backup with `pip freeze > requirements.txt` before deleting the environment | Medium |
| `environments.managers.mamba_env_remove` | Permanently removes a Mamba/Conda environment and all installed packages | Use `mamba env export` to save the environment specification first | High |
| `environments.managers.uv_cache_clean` | Removes the entire uv package cache | Use `uv cache dir` to inspect the cache location first | Medium |
| `environments.managers.micromamba_env_remove` | Permanently removes a Micromamba environment and all installed packages | Use `micromamba env export` to save the environment specification first | High |
| `environments.managers.poetry_env_remove` | Removes a Poetry virtual environment | Use `poetry env info` to review the environment first | Medium |
| `environments.managers.pipenv_rm` | Removes the Pipenv virtual environment | Ensure `Pipfile.lock` is committed so the environment can be recreated | Medium |
| `environments.managers.spack_uninstall_all` | Uninstalls all Spack packages, potentially removing shared dependencies | Uninstall specific packages by name; review with `spack find` first | High |

## R Language

**Pack ID:** `environments.r`

Guards against destructive R package management operations

### Safe patterns

| Rule ID | Pattern |
|---------|--------|
| `environments.r.r_version` | `^R\s+--version\b[^\|><]*$` |
| `environments.r.rscript_version` | `^Rscript\s+--version\b[^\|><]*$` |

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `environments.r.r_remove_packages` | Removes installed R packages | Use `installed.packages()` to review before removing | Medium |
| `environments.r.r_unlink` | Recursively deletes files or directories from an R script | Verify paths with `list.files()` before deleting | Medium |

---

This documentation was generated from [`rust/agents/src/tool_guard/shell/packs/environments.rs`](https://github.com/stencila/stencila/blob/main/rust/agents/src/tool_guard/shell/packs/environments.rs).
