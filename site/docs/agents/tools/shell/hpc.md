---
title: "HPC"
description: "Guards against destructive Singularity/Apptainer container operations. Guards against destructive HPC job scheduler operations"
---

This page lists the safe and destructive patterns in the **HPC Containers** and **HPC Schedulers** shell guard packs. See [Shell Tool](/docs/agents/tools/shell#guard-pipeline) for how these patterns are evaluated.

## HPC Containers

**Pack ID:** `hpc.apptainer`

Guards against destructive Singularity/Apptainer container operations

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `hpc.apptainer.singularity_delete` | Permanently removes a container image from a remote library | Use `singularity inspect` to review the image first | High |
| `hpc.apptainer.singularity_cache_clean_all` | Removes all cached container images, requiring re-download | Use `singularity cache list` to review cached images first | Medium |

## HPC Schedulers

**Pack ID:** `hpc.schedulers`

Guards against destructive HPC job scheduler operations

### Safe patterns

| Rule ID | Pattern |
|---------|--------|
| `hpc.schedulers.squeue` | `^squeue\b[^\|><]*$` |
| `hpc.schedulers.sinfo` | `^sinfo\b[^\|><]*$` |
| `hpc.schedulers.sacct` | `^sacct\b[^\|><]*$` |
| `hpc.schedulers.qstat` | `^qstat\b[^\|><]*$` |
| `hpc.schedulers.bjobs` | `^bjobs\b[^\|><]*$` |
| `hpc.schedulers.bhist` | `^bhist\b[^\|><]*$` |
| `hpc.schedulers.bqueues` | `^bqueues\b[^\|><]*$` |
| `hpc.schedulers.module_list` | `^module\s+list\b[^\|><]*$` |
| `hpc.schedulers.module_avail` | `^module\s+avail\b[^\|><]*$` |
| `hpc.schedulers.module_show` | `^module\s+show\b[^\|><]*$` |
| `hpc.schedulers.module_spider` | `^module\s+spider\b[^\|><]*$` |
| `hpc.schedulers.module_whatis` | `^module\s+whatis\b[^\|><]*$` |

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `hpc.schedulers.slurm_cancel_all` | Cancels all jobs for a user, including long-running computations | Cancel specific jobs by ID with `scancel <jobid>`; review with `squeue -u $USER` first | Medium |
| `hpc.schedulers.pbs_qdel_all` | Cancels all PBS/Torque queued and running jobs | Delete specific jobs by ID; review with `qstat` first | High |
| `hpc.schedulers.slurm_scontrol_shutdown` | Shuts down the Slurm controller daemon | This operation should not be performed by an agent | High |
| `hpc.schedulers.lsf_bkill_all` | Cancels all LSF jobs for the current user | Cancel specific jobs by ID with `bkill <jobid>`; review with `bjobs` first | High |
| `hpc.schedulers.lsf_bkill_user` | Cancels all LSF jobs for a user, including long-running computations | Cancel specific jobs by ID with `bkill <jobid>`; review with `bjobs -u $USER` first | Medium |
| `hpc.schedulers.sge_qdel_user` | Cancels all SGE/UGE jobs for a user | Delete specific jobs by ID; review with `qstat -u $USER` first | Medium |

---

This documentation was generated from [`rust/agents/src/tool_guard/shell/packs/hpc.rs`](https://github.com/stencila/stencila/blob/main/rust/agents/src/tool_guard/shell/packs/hpc.rs).
