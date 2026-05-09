---
title: "Provenance Summary Record"
description: "Compact projection of Stencila provenance categories."
---

# Provenance Summary Record

Compact projection of Stencila provenance categories.

The summary is derived from Stencila `ProvenanceCount` values. It uses a
single 0-100 percentage convention to avoid the previous ambiguity between
fractions and percentages.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`basis`](#basis) | `string` | No | Measurement basis for counts and percentages. |
| [`humanPercent`](#human-percent) | `number` | No | Percentage attributed to human-only provenance. |
| [`machinePercent`](#machine-percent) | `number` | No | Percentage attributed to machine-only provenance. |
| [`aiAssistedPercent`](#ai-assisted-percent) | `number` | No | Percentage attributed to mixed human and machine provenance. |
| [`source`](#source) | `string` | No | Source of the provenance summary. |
| [`sourceVersion`](#source-version) | `string` | No | Version of the summary source or algorithm. |
| [`categories`](#categories) | array<[`ProvenanceCategoryRecord`](provenance-category-record)> | No | Per-category provenance counts. |

### `basis`

Measurement basis for counts and percentages.

The default Stencila basis is `characters`. The field exists because
future summaries for images, tables, or multimodal outputs may use bytes,
pixels, cells, tokens, or another unit.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `humanPercent`

Percentage attributed to human-only provenance.

The value is a 0-100 percentage, not a fraction. It is optional because
consumers can always recompute it from categories when complete category
counts are disclosed.

**Type:** `number` | **Required:** No | **Nullable:** Yes

### `machinePercent`

Percentage attributed to machine-only provenance.

The value is a 0-100 percentage. This usually covers Stencila provenance
categories where machine generation or editing was present without human
editing or verification.

**Type:** `number` | **Required:** No | **Nullable:** Yes

### `aiAssistedPercent`

Percentage attributed to mixed human and machine provenance.

This value is a Stencila summary convenience, not a regulatory AI-use
declaration. Use `aiDisclosure` and standard C2PA AI disclosure for model
transparency.

**Type:** `number` | **Required:** No | **Nullable:** Yes

### `source`

Source of the provenance summary.

This should usually be `stencila-provenance-counts`. It is optional so
imported or externally computed summaries can identify their source.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `sourceVersion`

Version of the summary source or algorithm.

This is separate from the assertion payload version because provenance
counting algorithms may change without changing the wire schema.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `categories`

Per-category provenance counts.

Categories preserve the Stencila `ProvenanceCategory` vocabulary so
detailed consumers are not limited to coarse human/machine rollups.

**Type:** array<[`ProvenanceCategoryRecord`](provenance-category-record)> | **Required:** No


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
