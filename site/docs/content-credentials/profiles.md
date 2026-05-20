---
title: Provenance Profiles
description: Choose how much Stencila provenance to include in signed assets.
---

Profiles control how much Stencila provenance is included in a signed asset.
They are a privacy and disclosure choice, not a signature-strength setting. All
profiles still produce normal C2PA manifests and signatures.

> [!info]
> Choose a profile based on who will see the credential. Public, private, and
> full credentials can all be signed and verified; they differ in how much local
> context they disclose.

Profiles are independent of the [signing backend](signing). For example, a
public-profile credential can be signed locally or by Stencila Cloud:

```sh
stencila render article.smd --to pdf --credentials=public --credentials-signer local
stencila render article.smd --to pdf --credentials=public --credentials-signer cloud
```

## Choosing a Profile

Use this as a starting point:

| Profile | Best for | Main tradeoff |
|---------|----------|---------------|
| `public` | Journal submissions, preprints, repository deposits, public figures | Redacts local details that readers usually do not need |
| `private` | Sharing inside a research group or trusted collaboration | May expose paths, source details, or environment information |
| `full` | Controlled archives, reproducibility investigations, debugging | Preserves the most detail and should be treated as sensitive |

## Public

`public` is the default profile. It is intended for assets that may leave the
workspace or be uploaded to public verification tools.

The public profile keeps portable, low-risk facts such as asset type, media
type, producer version, public source references, and selected provenance
summary fields. It redacts local paths, private repository URLs, secret-looking
identifiers, and high-detail workflow state.

```sh
stencila render article.smd --to pdf --credentials
stencila render article.smd --to pdf --credentials=public
```

When signing rendered output, the default signer is `auto`: Stencila uses Cloud
signing when a Cloud token is available and falls back to local signing when
possible. Use `--credentials-signer local` when you need to avoid Cloud signing.

Use `public` for journal submission figures, preprints, repository deposits,
conference slides, and anything that may be checked by people outside your
team.

## Private

`private` is for internal sharing where collaborators may need more source and
execution context.

```sh
stencila render article.smd --to pdf --credentials=private
```

> [!warning]
> Private provenance can help collaborators understand how an asset was made,
> but it may also expose local paths, repository details, environment names, or
> workflow structure. Do not use it for public uploads unless you have reviewed
> the manifest.

## Full

`full` is for controlled archives and debugging. It preserves the most local
provenance detail Stencila currently knows how to project.

```sh
stencila render article.smd --to pdf --credentials=full
```

> [!warning]
> `full` is intentionally detailed. It is best suited to controlled archives,
> reproducibility investigations, and debugging, not routine public sharing.

Treat full-profile manifests as sensitive unless you have reviewed the signed
payload.

## Redactions

When fields are removed or summarized, Stencila records redaction metadata where
the active policy allows it. Verification output includes a compact redaction
count so reviewers can see whether the Stencila assertion is complete or
policy-filtered without reading the full payload.

> [!info]
> A redaction is not automatically suspicious. It usually means the selected
> profile withheld information that could be private, noisy, or unnecessary for
> the intended audience.

Profiles are additive in intent, but they are not a promise that every possible
local fact is captured. Some features, including exact reproducibility checks
and workflow attribution, are still deferred.
