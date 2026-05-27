# Stencila Content Credentials

This crate wraps the CAI `c2pa` Rust SDK to sign and verify assets with a
Stencila provenance assertion, `org.stencila.provenance`. The assertion payload
is a Stencila Schema `Graph`; standard C2PA assertions such as actions,
ingredients, and AI disclosure are projected from that graph. The crate exposes
the manual `stencila credentials ...` CLI path, Rust producer/verifier APIs,
and export-time signing through codec `EncodeOptions`.

## Current Scope

- Sign PNG, JPEG, WebP, SVG, and PDF assets with embedded C2PA manifests.
- Sign formats that Stencila does not embed directly with sidecar manifests.
- Verify embedded and sidecar manifests from Stencila and other producers.
- Report the four separate status axes from the design: manifest validity,
  signer trust, Stencila provenance attestation, and reproducibility status.
- Sign codec exports and extracted side assets when `EncodeOptions.credentials`
  is set, including source and ingredient provenance where available.
- Project the Stencila provenance graph into standard C2PA actions,
  ingredients, asset-type metadata, and AI disclosure where applicable.
- Generate local self-signed signing credentials that are visibly untrusted
  outside local or internal workflows.

Local signing credentials are useful for interoperability testing, but public
verifiers should show them as untrusted because they do not chain to a public
C2PA trust-list identity.

## Provenance Graph Payload

Stencila writes detailed provenance to the `org.stencila.provenance` assertion
as a Stencila Schema `Graph` with standalone JSON metadata:

```text
$schema: https://stencila.org/v<stencila-version>/Graph.schema.json
@context: https://stencila.org/v<stencila-version>/context.jsonld
```

The version segment follows the Stencila release that produced the credential.

The graph is the detailed Stencila provenance layer. It records signed assets,
document nodes, source files, executions, producer software, ingredients,
attributions, AI model use, reproducibility context, C2PA-derived provenance
from input assets, and privacy decisions where available. C2PA-facing manifest
fields are then derived from the same graph so generic C2PA tools can still see
portable actions, ingredients, asset metadata, and AI disclosure.

## Sidecar Convention

For sidecar manifests, this crate follows the CAI SDK convention: the sidecar
uses the same file stem as the asset with the extension replaced by `.c2pa`.
For example:

```text
animation.gif -> animation.c2pa
```

The SDK documentation describes this as a sidecar file with the same file name
as the asset but with a `.c2pa` extension. The verifier looks for embedded
manifests first and then for a nearby sidecar.

## Local Interoperability Test

These commands sign a local PNG with Stencila and then verify it locally.

```sh
cargo run --bin stencila -- credentials init

mkdir -p /tmp/stencila-c2pa
cargo run --bin stencila -- credentials sign \
  rust/content-credentials/tests/fixtures/sample.png \
  --output /tmp/stencila-c2pa/stencila-signed.png \
  --title "Stencila C2PA interop test"

cargo run --bin stencila -- credentials verify \
  /tmp/stencila-c2pa/stencila-signed.png

cargo run --bin stencila -- credentials inspect \
  /tmp/stencila-c2pa/stencila-signed.png \
  --as json
```

`inspect` prints manifest metadata. To also extract binary resources referenced
by that metadata, such as claim and ingredient thumbnails, pass a resources
directory:

```sh
cargo run --bin stencila -- credentials inspect \
  /tmp/stencila-c2pa/stencila-signed.png \
  --resources /tmp/stencila-c2pa/resources \
  --as yaml
```

The directory contains the extracted files and a `resources.json` index mapping
each file back to its C2PA resource identifier.

Expected local verification shape:

```text
Manifest valid:                  yes
Claim signature valid:           yes
Signer trusted:                  no   (Stencila local signing identity; local trust not configured)
Stencila provenance attested:    yes
Stencila reproducibility checked: not checked
```

`Signer trusted: no` is expected for `credentials init` certificates.

For certificate material that requires an external timestamp authority, pass
`--tsa-url` or set `STENCILA_CREDENTIALS_TSA_URL`:

```sh
cargo run --bin stencila -- credentials sign image.png \
  --cert /path/to/cert-or-chain.pem \
  --key /path/to/private-key.pem \
  --tsa-url https://tsa.example.org
```

To perform local signer-trust checks when verifying, Stencila uses the official
C2PA trust-list cache by default. The cache is refreshed automatically when
missing or stale. You can also refresh or inspect it explicitly:

```sh
cargo run --bin stencila -- credentials trust refresh
cargo run --bin stencila -- credentials trust status

cargo run --bin stencila -- credentials verify image.png
```

Use `--trust-anchors` or `STENCILA_CREDENTIALS_TRUST_ANCHORS` to override the
official cache with a local PEM bundle.

## Inspect With Content Credentials Verify

Use an embedded-image output such as PNG for web-tool testing. Sidecar workflows
are less portable through browser upload forms because the asset and sidecar must
travel together.

1. Run the local interoperability test above.
2. Open <https://contentcredentials.org/verify>.
3. Upload `/tmp/stencila-c2pa/stencila-signed.png`.
4. Confirm that the tool detects Content Credentials.
5. Confirm that the signer is shown as untrusted or unrecognized.
6. Inspect manifest details and check that the claim generator is Stencila.

Do not upload sensitive or private assets to external web tools unless that is
allowed by the relevant data policy.

## Verify Another Producer

To test reader interoperability, use an asset produced by another C2PA-capable
tool or device, such as Adobe Content Authenticity, a camera export, or
`c2patool`.

```sh
cargo run -p stencila-cli -- credentials verify path/to/other-producer.jpg

cargo run -p stencila-cli -- credentials inspect \
  path/to/other-producer.jpg \
  --as json
```

For a valid non-Stencila asset, the expected result is usually something like:

```text
Manifest valid:                  yes
Claim signature valid:           yes
Signer trusted:                  no   (OpenAI Media Service; local trust not configured)
Stencila provenance attested:    no   (assertion not present)
Stencila reproducibility checked: not checked
```

`Stencila provenance attested: no` only means the asset does not carry Stencila's
`org.stencila.provenance` assertion. It can still be a valid C2PA asset.

If the asset uses a sidecar manifest, keep the `.c2pa` file next to the asset
with the same stem, for example `photo.jpg` and `photo.c2pa`.

## Conformance Interoperability Evidence

For C2PA conformance prep, use the scripts in
[`interop/`](interop/README.md). They produce evidence directories containing
tool versions, asset digests, Stencila verification output, Stencila inspection
JSON, and `c2patool` output.

```sh
rust/content-credentials/interop/stencila-to-c2patool.sh
rust/content-credentials/interop/other-to-stencila.sh path/to/other-producer.jpg
```

## Useful References

- C2PA specification: <https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html>
- CAI Rust SDK documentation: <https://opensource.contentauthenticity.org/docs/rust-sdk/>
- CAI manifest overview: <https://opensource.contentauthenticity.org/docs/manifest/understanding-manifest/>
- Content Credentials Verify: <https://contentcredentials.org/verify>
