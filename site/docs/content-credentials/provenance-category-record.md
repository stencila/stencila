---
title: "Provenance Category Record"
description: "Per-category count for a Stencila provenance category."
---

# Provenance Category Record

Per-category count for a Stencila provenance category.

This mirrors `ProvenanceCount` closely enough that a consumer can map it back
to Stencila concepts without loading the full generated Stencila Schema.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`provenanceCategory`](#provenance-category) | `string` | Yes | Stencila provenance category. |
| [`characterCount`](#character-count) | `integer` | Yes | Number of counted units in the category. |
| [`characterPercent`](#character-percent) | `number` | No | Percentage of counted content in this category. |

### `provenanceCategory`

Stencila provenance category.

Values should come from `ProvenanceCategory`, such as `Hw`, `HwMv`,
`MwHe`, or `MwMeMv`. The legacy `category` field name is accepted for
unpublished payloads.

**Type:** `string` | **Required:** Yes

### `characterCount`

Number of counted units in the category.

The unit is defined by `ProvenanceSummaryRecord.basis`, usually
characters. This field is required because percentages alone are not
enough to compare summaries across differently sized works.

**Type:** `integer` | **Required:** Yes

### `characterPercent`

Percentage of counted content in this category.

The value is a 0-100 percentage, matching Stencila
`ProvenanceCount.characterPercent`.

**Type:** `number` | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
