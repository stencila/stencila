---
title: Sidecar Credentials
description: Understand detached `.c2pa` sidecar files for signed assets.
---

Some file formats can carry a Content Credential inside the file. Others cannot.
For those assets, Stencila writes a detached `.c2pa` sidecar next to the asset.

> [!info]
> A sidecar is a separate credential file for an asset. It is still
> cryptographically bound to the asset bytes; it is stored beside the asset
> instead of inside it.

## When Sidecars Are Used

PNG, JPEG, WebP, and SVG receive embedded manifests by default. For PDF and
other formats that cannot be embedded by the C2PA SDK, Stencila writes a
sidecar.

Browser-based verification tools often work best with embedded manifests because
single-file upload forms may not preserve sidecars. For manual web checks, use
an embedded format such as PNG unless the tool explicitly supports sidecar
uploads.

## Naming

Sidecars use the same file stem as the signed asset with a `.c2pa` extension:

```text
asset.ext -> asset.c2pa
report.pdf -> report.c2pa
```

> [!tip]
> When you see `report.pdf` and `report.c2pa` together, treat them as one
> signed package. The PDF is what people read; the sidecar is where the Content
> Credential lives.

## Copying Assets

Keep the asset and sidecar together. If a signed PDF is copied without its
`.c2pa` file, verifiers will not find the manifest.

When sending sidecar-backed assets to another person or system, copy both files:

```text
report.pdf
report.c2pa
```

If you rename the asset, rename the sidecar stem to match:

```text
report-v2.pdf
report-v2.c2pa
```

> [!warning]
> Sending only the visible file is enough for a reader to open it, but not
> enough for a verifier to inspect its sidecar-backed Content Credential.

## Verification

Stencila looks for embedded manifests first when the media type supports them.
If no embedded manifest is found, it then looks for a nearby sidecar using the
same naming convention.

For sidecar-backed assets, `stencila credentials verify` reports the manifest as
coming from a sidecar.

> [!tip]
> For quick demonstrations or manual checks in a browser, a signed PNG is often
> simpler than a signed PDF because the credential can be embedded in the same
> file.
