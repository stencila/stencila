---
title: "System"
description: "Guards against destructive disk operations. Guards against destructive network operations. Guards against destructive system service operations"
---

This page lists the safe and destructive patterns in the **Disk**, **Network**, and **Services** shell guard packs. See [Shell Tool](/docs/agents/tools/shell#guard-pipeline) for how these patterns are evaluated.

## Disk

**Pack ID:** `system.disk`

Guards against destructive disk operations

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `system.disk.dd_to_device` | Writing directly to devices can destroy partitions and data | Specify a file path instead of a device | High |
| `system.disk.mkfs` | Formatting a device destroys all data on it | Verify the target device with `lsblk` first | High |
| `system.disk.fdisk_parted` | Partition changes can cause data loss | Use `fdisk -l` or `parted print` to inspect first | Medium |

## Network

**Pack ID:** `system.network`

Guards against destructive network operations

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `system.network.iptables_flush` | Flushing firewall rules removes all network security policies | Use `iptables -L` to list rules first; save with `iptables-save` | High |
| `system.network.route_delete` | Deleting routes can cause network connectivity loss | Use `ip route show` to review routes before modification | Medium |
| `system.network.interface_down` | Bringing down a network interface disrupts connectivity | Ensure you have alternative access before modifying interfaces | Medium |

## Services

**Pack ID:** `system.services`

Guards against destructive system service operations

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `system.services.systemctl_destructive` | Stopping or disabling critical services can break the system | Use `systemctl status` to check service state first | Medium |
| `system.services.service_stop` | Stopping services can disrupt running applications | Use `service <name> status` to check before stopping | Medium |
| `system.services.kill_signal` | SIGKILL terminates processes without cleanup | Use `kill` (SIGTERM) first to allow graceful shutdown | Medium |

---

This documentation was generated from [`rust/agents/src/tool_guard/shell/packs/system.rs`](https://github.com/stencila/stencila/blob/main/rust/agents/src/tool_guard/shell/packs/system.rs).
