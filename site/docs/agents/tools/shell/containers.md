---
title: "Containers"
description: "Guards against destructive Docker operations. Guards against destructive Kubernetes operations"
---

This page lists the safe and destructive patterns in the **Docker** and **Kubernetes** shell guard packs. See [Shell Tool](/docs/agents/tools/shell#guard-pipeline) for how these patterns are evaluated.

## Docker

**Pack ID:** `containers.docker`

Guards against destructive Docker operations

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `containers.docker.system_prune` | Removes all unused containers, networks, images, and optionally volumes | Use `docker container prune` or `docker image prune` for targeted cleanup | High |
| `containers.docker.volume_prune` | Permanently deletes all unused volumes and their data | List volumes with `docker volume ls` and remove specific ones | High |
| `containers.docker.force_remove` | Force-removes running containers or in-use images | Stop containers first with `docker stop`, then remove | Medium |

## Kubernetes

**Pack ID:** `containers.kubectl`

Guards against destructive Kubernetes operations

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `containers.kubectl.delete_namespace` | Deletes all resources in the namespace | Delete specific resources within the namespace instead | High |
| `containers.kubectl.delete_all` | Mass-deletes resources across scopes | Delete specific resources by name | High |
| `containers.kubectl.drain_node` | Evicts all pods from a node | Use `kubectl drain --dry-run=client` first to preview | Medium |

---

This documentation was generated from [`rust/agents/src/tool_guard/shell/packs/containers.rs`](https://github.com/stencila/stencila/blob/main/rust/agents/src/tool_guard/shell/packs/containers.rs).
