---
title: Shell Tool
description: Tool for executing shell commands, and the pattern-based guard system that evaluates every command before execution.
---

The shell tool allows agents to execute commands in a bash shell. Each command is evaluated by the [shell guard](#guard-pipeline) before execution.

## shell

Executes a shell command and returns its output.

| Parameter | Type | Required | Description |
| --------- | ---- | :------: | ----------- |
| `command` | string | ✅ | The shell command to execute |
| `timeout_ms` | integer (min: 1) | | Timeout in milliseconds. Defaults to 10000 (10 seconds) |
| `description` | string | | Human-readable description of what the command does (for logging only) |

The per-call timeout is clamped to a provider-specific maximum (e.g., 10 minutes). Output includes the exit code, duration, and stdout/stderr content. If the command exceeds the timeout, it is killed and the partial output is returned with a `[TIMED OUT]` indicator.

## Guard Pipeline

The shell guard evaluates commands using a multi-step pipeline:

1. **Command extraction** — the raw command string is parsed to extract individual segments (splitting on `&&`, `||`, `;`, etc.) and command substitutions. Wrapper commands (`bash -c`, `sh -c`, etc.) are recursively unwrapped up to 5 levels of nesting. Commands longer than 8,192 bytes are rejected.

2. **Safe pattern check** — each segment is tested against the combined set of safe patterns from all [guard packs](#guard-packs). If a segment matches a safe pattern, it is considered safe and skips destructive checking.

3. **Destructive pattern check** — segments that did not match any safe pattern are tested against destructive patterns. A match produces a `Deny` or `Warn` verdict depending on the pattern's confidence level and the session's trust level.

4. **Default verdict** — segments that match neither safe nor destructive patterns receive the default verdict based on trust level:
   - **Low trust**: `Deny` (default-deny)
   - **Medium / High trust**: `Allow` (default-allow)

The strictest verdict across all segments becomes the final verdict for the command.

### Command Extraction

The extraction pipeline handles:

- **Shell wrappers** — `bash -c "..."`, `sh -c "..."`, `zsh -c "..."`, and Windows equivalents (`cmd /c`, `powershell -command`) are unwrapped recursively, including path-qualified variants like `/usr/bin/bash`
- **Environment wrappers** — `env` prefixes (with flags like `-u`, `-C`, `-S`, `-i`, `VAR=val`) are stripped to expose the inner command
- **Compound commands** — splitting on `&&`, `||`, `;`, `&`, and newlines
- **Command substitutions** — `$(...)` and backtick expressions are extracted from unquoted and double-quoted contexts (but not single-quoted strings)
- **Sudo/doas** — stripped and the inner command re-evaluated; at low trust, `sudo`/`doas` alone is denied
- **Pipe segments** — passed individually to destructive pattern validators

### Trust Level Behavior

| Trust Level | Default (no match) | Destructive (high confidence) | Destructive (medium confidence) |
| ----------- | ------------------ | ----------------------------- | ------------------------------- |
| Low | Deny | Deny | Deny |
| Medium | Allow | Deny | Deny |
| High | Allow | Deny | Warn |

## Guard Packs

Shell guard patterns are organized into **packs** — groups of related safe and destructive patterns. Each pack targets a specific domain (filesystem operations, git, cloud services, etc.).

<!-- PACKS_TABLE_START -->
| Pack ID | Name | Description | Safe Patterns | Destructive Patterns |
|---------|------|-------------|:-------------:|:--------------------:|
| `core.filesystem` | [Filesystem](core#filesystem) | Guards against recursive/forced file deletion and dangerous moves/overwrites | 42 | 10 |
| `core.git` | [Git](core#git) | Guards against destructive git operations that lose history or modify remote state | 8 | 11 |
| `core.obfuscation` | [Obfuscation](core#obfuscation) | Guards against meta-execution patterns whose purpose is guard bypass | 0 | 5 |
| `core.stencila` | [Stencila](core#stencila) | Guards the agent's own runtime, credentials, and publishing operations | 9 | 8 |
| `bioinformatics.sequence_tools` | [Sequence Tools](bioinformatics#sequence-tools) | Guards against destructive bioinformatics sequence analysis operations | 17 | 4 |
| `chemistry.molecular_dynamics` | [Molecular Dynamics](chemistry#molecular-dynamics) | Guards against destructive molecular dynamics and chemistry tool operations | 3 | 3 |
| `cloud.aws` | [AWS](cloud#aws) | Guards against destructive AWS operations | 0 | 4 |
| `cloud.azure` | [Azure](cloud#azure) | Guards against destructive Azure operations | 0 | 3 |
| `cloud.gcp` | [Google Cloud](cloud#google-cloud) | Guards against destructive GCP operations | 0 | 3 |
| `cloud.iac` | [Infrastructure as Code](cloud#infrastructure-as-code) | Guards against destructive IaC operations | 0 | 2 |
| `containers.docker` | [Docker](containers#docker) | Guards against destructive Docker operations | 0 | 3 |
| `containers.kubectl` | [Kubernetes](containers#kubernetes) | Guards against destructive Kubernetes operations | 0 | 3 |
| `database.mysql` | [MySQL](database#mysql) | Guards against destructive MySQL operations | 0 | 4 |
| `database.postgresql` | [PostgreSQL](database#postgresql) | Guards against destructive PostgreSQL operations | 0 | 4 |
| `database.sqlite` | [SQLite](database#sqlite) | Guards against destructive SQLite operations | 0 | 2 |
| `datasets.versioning` | [Data Versioning](datasets#data-versioning) | Guards against destructive operations on scientific data versioning tools | 8 | 5 |
| `datasets.transfer` | [Data Transfer Tools](datasets#data-transfer-tools) | Guards against destructive operations in scientific data transfer tools | 5 | 5 |
| `environments.managers` | [Environment Managers](environments#environment-managers) | Guards against destructive operations on environment and package managers | 15 | 10 |
| `environments.r` | [R Language](environments#r-language) | Guards against destructive R package management operations | 2 | 2 |
| `geospatial.climate_data` | [Climate Data Tools](geospatial#climate-data-tools) | Guards against destructive operations on NetCDF/HDF climate data files | 2 | 3 |
| `geospatial.gdal` | [GDAL/OGR](geospatial#gdal-ogr) | Guards against destructive GDAL/OGR geospatial data operations | 2 | 4 |
| `hpc.apptainer` | [HPC Containers](hpc#hpc-containers) | Guards against destructive Singularity/Apptainer container operations | 0 | 2 |
| `hpc.schedulers` | [HPC Schedulers](hpc#hpc-schedulers) | Guards against destructive HPC job scheduler operations | 12 | 6 |
| `latex.build_tools` | [LaTeX Build Tools](latex#latex-build-tools) | Guards against destructive LaTeX build tool operations | 2 | 2 |
| `ml.experiment_tracking` | [ML Experiment Tracking](ml#ml-experiment-tracking) | Guards against destructive ML experiment and model tracking operations | 4 | 5 |
| `notebooks.jupyter` | [Jupyter Notebooks](notebooks#jupyter-notebooks) | Guards against destructive Jupyter notebook operations | 1 | 3 |
| `packages.registries` | [Package Registries](packages#package-registries) | Guards against destructive package manager operations affecting registries | 0 | 5 |
| `scientific.computing` | [Scientific Computing](scientific#scientific-computing) | Guards against destructive operations in scientific computing environments | 2 | 5 |
| `system.disk` | [Disk](system#disk) | Guards against destructive disk operations | 0 | 3 |
| `system.network` | [Network](system#network) | Guards against destructive network operations | 0 | 3 |
| `system.services` | [Services](system#services) | Guards against destructive system service operations | 0 | 3 |
| `workflows.engines` | [Workflow Engines](workflows#workflow-engines) | Guards against destructive operations in scientific workflow engines | 4 | 4 |
<!-- PACKS_TABLE_END -->
