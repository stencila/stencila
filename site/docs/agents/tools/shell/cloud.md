---
title: "Cloud"
description: "Guards against destructive AWS operations. Guards against destructive Azure operations. Guards against destructive GCP operations. Guards against destructive IaC operations"
---

This page lists the safe and destructive patterns in the **AWS**, **Azure**, **Google Cloud**, and **Infrastructure as Code** shell guard packs. See [Shell Tool](/docs/agents/tools/shell#guard-pipeline) for how these patterns are evaluated.

## AWS

**Pack ID:** `cloud.aws`

Guards against destructive AWS operations

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `cloud.aws.terminate_instances` | Permanently destroys EC2 instances | Use `aws ec2 stop-instances` to stop without terminating | High |
| `cloud.aws.delete_db` | Permanently deletes database instances | Create a final snapshot first with `--final-db-snapshot-identifier` | High |
| `cloud.aws.s3_recursive_delete` | Recursively deletes S3 objects or force-removes buckets | Use `aws s3 ls` to inspect first; delete specific prefixes | High |
| `cloud.aws.iam_delete` | Removes IAM identities and their permissions | Use `aws iam list-*` to review before deletion | Medium |

## Azure

**Pack ID:** `cloud.azure`

Guards against destructive Azure operations

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `cloud.azure.vm_delete` | Permanently destroys virtual machines | Use `az vm deallocate` to stop without deleting | High |
| `cloud.azure.group_delete` | Deletes a resource group and all resources within it | Use `az group show` to review contents first | High |
| `cloud.azure.storage_delete` | Deletes storage blobs or containers | Use `az storage blob list` to review contents first | Medium |

## Google Cloud

**Pack ID:** `cloud.gcp`

Guards against destructive GCP operations

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `cloud.gcp.compute_delete` | Permanently destroys Compute Engine instances | Use `gcloud compute instances stop` to stop without deleting | High |
| `cloud.gcp.storage_delete` | Recursively deletes Cloud Storage objects or force-removes buckets | Use `gsutil ls` to inspect first; delete specific objects | High |
| `cloud.gcp.sql_delete` | Permanently deletes Cloud SQL instances | Create a backup first with `gcloud sql backups create` | High |

## Infrastructure as Code

**Pack ID:** `cloud.iac`

Guards against destructive IaC operations

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `cloud.iac.terraform_destroy` | Destroys all managed infrastructure resources | Use `terraform plan -destroy` to preview what will be destroyed | High |
| `cloud.iac.pulumi_destroy` | Destroys all managed infrastructure resources | Use `pulumi preview --diff` to review changes first | High |

---

This documentation was generated from [`rust/agents/src/tool_guard/shell/packs/cloud.rs`](https://github.com/stencila/stencila/blob/main/rust/agents/src/tool_guard/shell/packs/cloud.rs).
