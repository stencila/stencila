---
title: Using Content Credentials
description: Sign, verify, inspect, and manage trust for Stencila Content Credentials.
---

Stencila can sign exported assets with Content Credentials and verify
credentials produced by Stencila or other C2PA tools.

Signing adds provenance to an asset. Verifying checks whether that provenance is
still attached, still matches the asset, and, when possible, was signed by a
signer your verifier recognizes.

The commands live under `stencila credentials`:

```sh
stencila credentials init
stencila credentials sign figure.png
stencila credentials verify figure.png
stencila credentials inspect figure.png
stencila credentials trust status
```

> [!tip]
> If you are reading or reviewing someone else's file, start with `verify`.
> Then use `inspect` when you need to see the manifest details.

Rendered outputs can also be signed directly:

```sh
stencila render article.smd article.pdf --credentials
stencila render article.smd article.pdf --credentials=private
stencila render article.smd article.pdf --credentials --credentials-signer cloud
```

`--credentials` selects the provenance [profile](profiles). `--credentials-signer`
selects the [signing backend](signing).

You can also set workspace defaults in `stencila.toml`:

```toml
[content-credentials]
enabled = true
profile = "public"
signer = "auto"
```

## Common Workflows

For a first local test:

```sh
stencila credentials init
stencila credentials sign figure.png
stencila credentials verify figure.png
```

For a figure, report, or archive file you plan to share:

```sh
stencila credentials sign figure.png --output signed-figure.png
stencila credentials verify signed-figure.png
```

For a publication or repository workflow that requires Stencila provenance:

```sh
stencila credentials verify figure.png --require stencila-assertion
```

For a render that should use Stencila Cloud signing when available:

```sh
stencila cloud signin
stencila render article.smd article.pdf --credentials
```

Or require Cloud signing for a workspace:

```toml
[content-credentials]
enabled = true
signer = "cloud"
```

## Initialize a Local Signer

Use `init` to create a local signing identity:

```sh
stencila credentials init
```

This identity is self-signed. It is useful for local testing and interoperability
evidence, but public verifiers should usually show it as untrusted.

> [!info]
> "Self-signed" means the signing identity was created locally and is not
> vouched for by a public certificate authority or C2PA trust list. The
> credential can still be intact and useful inside a team, but other verifiers
> may not recognize the signer.

For production signing of an existing asset with local certificate material,
pass the certificate and key explicitly:

```sh
stencila credentials sign figure.png \
  --cert /path/to/cert-or-chain.pem \
  --key /path/to/private-key.pem \
  --tsa-url https://tsa.example.org
```

## Sign an Asset

Sign an existing asset in place:

```sh
stencila credentials sign figure.png
```

This standalone signing command uses the local signing backend. Rendered
outputs use automatic signing by default; use `stencila render
--credentials-signer cloud` when local fallback should be an error.

Write the signed result to another path:

```sh
stencila credentials sign figure.png --output signed-figure.png
```

PNG, JPEG, WebP, SVG, and PDF receive embedded manifests. Formats that cannot
be embedded by Stencila or the C2PA SDK receive a `.c2pa` sidecar next to the
asset.

> [!warning]
> If an asset is edited after signing, verify it again before sharing it. For
> sidecar-backed formats, keep the visible file and its `.c2pa` sidecar
> together.

For machine-readable output, use `--as`:

```sh
stencila credentials sign figure.png --as json
```

Structured sign output includes the signed path, manifest kind, manifest ID
when available, sidecar path, profile, signing mode, digests, and non-fatal
warnings.

## Verify an Asset

Verify an asset:

```sh
stencila credentials verify figure.png
```

The default output separates four questions:

- Is there a C2PA manifest, and is it well formed?
- Does the signed claim still match the asset?
- Does the verifier recognize the signer?
- Is there Stencila provenance, and what reproducibility status was recorded?

> [!info]
> These are related but distinct checks. A credential can be intact even when
> the signer is not recognized by your current verifier. A recognized signer
> does not prove the scientific result is correct.

Use strict requirements when a workflow needs a specific property:

```sh
stencila credentials verify figure.png --require stencila-assertion
stencila credentials verify figure.png --require trusted-signer
```

`--require repro-exact` is reserved in v1 and reports an unavailable problem
because reproducibility comparisons are not implemented yet.

> [!tip]
> Use `--require stencila-assertion` when a workflow specifically needs Stencila
> document provenance. Use `--require trusted-signer` when your policy requires
> a recognized signing identity.

Use structured output for CI and evidence collection:

```sh
stencila credentials verify figure.png --as json
```

## Inspect the Manifest

Inspect prints the full C2PA manifest:

```sh
stencila credentials inspect figure.png
```

The default format is YAML for readability. Use `--as json` when comparing with
other tooling or collecting evidence.

For Stencila-created manifests, inspect output includes `claim_generator_info`.
That section identifies whether the provenance was generated locally, signed
locally, signed by Stencila Cloud, or generated and signed by Stencila Cloud.

> [!warning]
> Full manifests can contain local paths, source details, environment
> information, or other provenance that is more detailed than a reader-facing
> summary. Review the active [profile](profiles) before sharing inspected
> payloads or logs.

## Manage Trust Lists

Stencila uses the official C2PA trust-list cache by default for local signer
trust checks. Refresh and inspect it with:

```sh
stencila credentials trust refresh
stencila credentials trust status
```

Use `--trust-anchors` or `STENCILA_CREDENTIALS_TRUST_ANCHORS` only when a local
workflow needs a custom PEM bundle instead of the official cache.

See [Trust](trust) for the difference between a valid signature and a signer
recognized by your verifier.
