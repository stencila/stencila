---
title: "Privacy Record"
description: "Privacy signals and redactions applied while building the assertion."
---

# Privacy Record

Privacy signals and redactions applied while building the assertion.

This record says what the projection policy did and what it assessed. It uses
structured statuses instead of booleans so a manifest can distinguish
`not-assessed` from `none-detected`.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`redactions`](#redactions) | array<[`RedactionRecord`](redaction-record)> | No | Redactions applied while building the assertion. |
| [`personalData`](#personal-data) | [`DisclosureAssessmentRecord`](disclosure-assessment-record) | Yes | Personal-data assessment for disclosed fields. |
| [`secrets`](#secrets) | [`DisclosureAssessmentRecord`](disclosure-assessment-record) | Yes | Secret assessment for disclosed fields. |

### `redactions`

Redactions applied while building the assertion.

Each redaction identifies a field path and a reason. The list is empty
when no fields were redacted or when the policy chooses not to disclose
redaction details.

**Type:** array<[`RedactionRecord`](redaction-record)> | **Required:** No

### `personalData`

Personal-data assessment for disclosed fields.

A structured status avoids claiming "no personal data" unless a policy or
scanner actually assessed it.

**Type:** [`DisclosureAssessmentRecord`](disclosure-assessment-record) | **Required:** Yes

### `secrets`

Secret assessment for disclosed fields.

A structured status lets producers distinguish redaction, non-disclosure,
no scan, and no secret detected.

**Type:** [`DisclosureAssessmentRecord`](disclosure-assessment-record) | **Required:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
