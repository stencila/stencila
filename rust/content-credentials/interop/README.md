# Content Credentials Interoperability Evidence

This directory contains a local evidence harness for C2PA conformance prep. It
is intentionally not a CI gate: the scripts collect tool versions, commands,
asset digests, Stencila verifier output, and `c2patool` validator output into a
packet that can be reviewed or attached to a conformance application.

The main workflow proves two interoperability paths:

- Stencila as Generator, with `c2patool` as Validator.
- Another C2PA Generator, with Stencila as Validator.

## Prerequisites

Required:

- Stencila CLI from this workspace, usually run as `cargo run --bin stencila -- ...`.
- `c2patool`, the CAI command-line implementation.

Install `c2patool` from the official docs:

- <https://opensource.contentauthenticity.org/docs/c2patool/>

Optional manual check:

- Content Credentials Verify: <https://contentcredentials.org/verify>

## Signing Setup

For local development evidence, initialize a self-signed development identity:

```sh
cargo run --bin stencila -- credentials init
```

Development certificates produce valid signatures but are intentionally
untrusted by public verifiers. For conformance or production evidence, provide
real signing material instead:

```sh
export STENCILA_CREDENTIALS_CERT=/path/to/cert-or-chain.pem
export STENCILA_CREDENTIALS_KEY=/path/to/private-key.pem
export STENCILA_CREDENTIALS_TSA_URL=https://tsa.example.org
```

Stencila uses the official C2PA trust-list cache for local signer-trust checks
by default. Refresh it before an evidence run when you want to record the exact
trust-list material used:

```sh
cargo run --bin stencila -- credentials trust refresh
cargo run --bin stencila -- credentials trust status --as json
```

Use `STENCILA_CREDENTIALS_TRUST_ANCHORS` only when you need to override the
official cache with a local PEM bundle. Trust will only become `yes` if the
asset's signing chain traces to one of the loaded anchors.

## Run Evidence

Run the full local evidence packet:

```sh
rust/content-credentials/interop/run-evidence.sh
```

Default output:

```text
target/content-credentials-interop/evidence/
```

The wrapper signs and validates the in-scope Stencila fixtures:

| Format | Fixture | Expected signing mode |
| --- | --- | --- |
| PNG | `tests/fixtures/sample.png` | Embedded |
| JPEG | `tests/fixtures/sample.jpg` | Embedded |
| WebP | `tests/fixtures/sample.webp` | Embedded |
| SVG | `tests/fixtures/sample.svg` | Embedded |
| PDF | `tests/fixtures/sample.pdf` | Sidecar |

To include reverse validator evidence with a third-party signed sample, pass the
asset as the second argument or set `THIRD_PARTY_ASSET`:

```sh
rust/content-credentials/interop/run-evidence.sh \
  target/content-credentials-interop/evidence \
  path/to/third-party-signed.jpg
```

If the third-party asset uses a sidecar manifest, keep the `.c2pa` file next to
the asset using the CAI sidecar convention, for example `photo.jpg` and
`photo.c2pa`.

## Evidence Layout

The wrapper writes a top-level manifest:

```text
target/content-credentials-interop/evidence/MANIFEST.txt
```

`MANIFEST.txt` records:

- generation timestamp
- Stencila and `c2patool` versions
- signing configuration
- fixture digests
- per-format pass/fail rows

Stencila Generator -> `c2patool` Validator evidence is written under:

```text
target/content-credentials-interop/evidence/stencila-to-c2patool/<format>/
```

Each format directory contains:

- `tool-versions.txt`
- `digests.txt`
- `stencila-init.txt`
- `stencila-sign.txt`
- `stencila-verify.txt`
- `stencila-verify.json`
- `stencila-inspect.json`
- `c2patool-summary.json`
- `c2patool-detailed.json`
- `c2patool-trust.json` when `C2PATOOL_TRUST_ANCHORS` is set

Other Generator -> Stencila Validator evidence is written under:

```text
target/content-credentials-interop/evidence/other-to-stencila/
```

That directory contains the same verifier and digest files, excluding
`stencila-init.txt` and `stencila-sign.txt` because Stencila did not generate
the asset.

## Expected Results

For Stencila-generated fixtures, the expected result is:

- Stencila verification reports a present, valid manifest.
- Claim signature validation succeeds.
- Asset binding validation succeeds.
- `org.stencila.provenance` is present and attested.
- `c2patool --detailed` reports the standard C2PA assertions without
  conformance errors.

With local development certificates, signer trust is expected to be false or
unrecognized. That is not an interoperability failure. Public trust-list
recognition requires suitable C2PA signing material.

For a valid non-Stencila asset, Stencila should report the C2PA manifest and
asset binding status cleanly. `org.stencila.provenance` will usually be absent,
which only means the asset was not produced by Stencila.

## Lower-Level Scripts

Use these scripts when debugging a single direction or format.

### Stencila Generator, c2patool Validator

```sh
rust/content-credentials/interop/stencila-to-c2patool.sh \
  rust/content-credentials/tests/fixtures/sample.png \
  target/content-credentials-interop/stencila-to-c2patool/png
```

This signs one input asset with Stencila, verifies it with Stencila, and records
`c2patool` summary and detailed output.

### Other Generator, Stencila Validator

```sh
rust/content-credentials/interop/other-to-stencila.sh \
  path/to/signed-asset.jpg \
  target/content-credentials-interop/other-to-stencila
```

This verifies a third-party C2PA asset with Stencila and records matching
`c2patool` output for comparison.

## Manual Browser Check

For browser-tool evidence, upload a Stencila-signed embedded asset, such as the
PNG output from the evidence packet, to <https://contentcredentials.org/verify>.

With the development certificate, expect the issuer to be unrecognized. With
conformance signing material, expect the issuer result to reflect the public
trust-list status of that certificate.

Do not upload sensitive or private assets to external web tools unless that is
allowed by the relevant data policy.

## Known Limits

- Reverse validator evidence is skipped unless a third-party signed asset is
  supplied. A redistributable bundled third-party sample is tracked separately.
- Local development certificates are intentionally untrusted by public
  verifiers.
- Public trust-list recognition depends on a suitable C2PA signing certificate
  and the verifier's current trust-list configuration.
