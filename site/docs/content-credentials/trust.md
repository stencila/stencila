---
title: Trust and Signers
description: Understand valid signatures, recognized signers, and trust policies.
---

When you verify a signed asset, Stencila answers two related but different kinds
of questions:

1. Has the asset or its Content Credential changed since it was signed?
2. Does your verifier recognize the person, organization, device, or software
   that signed it?

The first question is about integrity. If verification says the signature and
asset binding are valid, the signed content still matches the Content
Credential attached to it.

The second question is about trust policy. A valid signature tells you that the
credential has not been tampered with. It does not automatically tell you that
the signer is known, reputable, authorized, or appropriate for your review. For
that, a verifier checks whether the signer's certificate connects to a trust
list or trust anchor that the verifier accepts.

For example, a colleague can sign an image with a local Stencila signing
identity. The credential can be intact and useful for sharing provenance inside
your team, but a public verification service may still report the signer as
untrusted because it has no independent reason to recognize that local identity.

> [!info]
> Think of signing like sealing a lab notebook entry. The seal can show whether
> the entry changed after signing. Trust policy is the separate question of
> whether your workflow recognizes the person or organization that applied the
> seal.

In practice:

- **Valid** means the Content Credential still matches the signed asset.
- **Trusted** means the signer is recognized under the verifier's trust policy.
- **Untrusted** does not necessarily mean fake or broken. It often means the
  signer is local, private, experimental, expired, or outside the verifier's
  current trust list.

> [!warning]
> Do not collapse "untrusted" into "false". An untrusted signer status is a
> reason to ask who signed the asset and which trust policy was used, not proof
> that the content was fabricated.

## Local Signers

`stencila credentials init` creates a self-signed local identity:

```sh
stencila credentials init
```

Use it for local testing and private workflows. Public verifiers should show
this signer as untrusted or unrecognized because it does not chain to a public
C2PA trust-list identity.

> [!tip]
> Local signers are useful while developing a workflow, teaching provenance, or
> exchanging files inside a small team. For public release, use a signing
> identity that your intended verifiers are expected to recognize.

## Official Trust List

Stencila uses the official C2PA trust-list cache by default for local trust
checks. Refresh and inspect the cache with:

```sh
stencila credentials trust refresh
stencila credentials trust status
```

Trust-list recognition depends on the asset's signing certificate chain and the
verifier's current trust-list configuration.

> [!info]
> Different verifiers may use different trust policies or refresh their trust
> lists at different times. When results differ, compare the trust configuration
> as well as the asset.

## Custom Trust Anchors

For local or internal PKI workflows, provide a PEM bundle explicitly:

```sh
stencila credentials verify image.png --trust-anchors anchors.pem
```

The same override is available through `STENCILA_CREDENTIALS_TRUST_ANCHORS`.
Only use custom anchors when you intentionally want to replace the official
trust-list cache for that verification run.

> [!warning]
> A custom trust anchor changes who your local verifier recognizes. Use it for a
> deliberate institutional or project policy, not as a way to make an
> unexpected verification result disappear.

## Production Signing

For production signing, use certificate material appropriate for that workflow
and pass a timestamp authority if required:

```sh
stencila credentials sign image.png \
  --cert /path/to/cert-or-chain.pem \
  --key /path/to/private-key.pem \
  --tsa-url https://tsa.example.org
```

Keep development-signed assets clearly labeled so reviewers do not interpret
expected untrusted signer status as a manifest failure.
