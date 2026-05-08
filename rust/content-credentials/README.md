# Stencila Content Credentials

This crate wraps the CAI `c2pa` Rust SDK to sign and verify assets with a
Stencila custom assertion, `org.stencila.provenance`. It currently exposes the
manual `stencila credentials ...` CLI path and the Rust producer/verifier APIs.
Render and export integration through codec `EncodeOptions` is a later slice.

## Current Scope

- Sign PNG, JPEG, WebP, and SVG assets with embedded C2PA manifests.
- Sign formats that cannot be embedded by the SDK, including PDF, with sidecar
  manifests.
- Verify embedded and sidecar manifests from Stencila and other producers.
- Report the four separate status axes from the design: manifest validity,
  signer trust, Stencila provenance attestation, and reproducibility status.
- Generate local self-signed development credentials that are visibly untrusted
  outside local testing.

Development credentials are useful for interoperability testing, but public
verifiers should show them as untrusted because they do not chain to a public
C2PA trust-list identity.

## Sidecar Convention

For sidecar manifests, this crate follows the CAI SDK convention: the sidecar
uses the same file stem as the asset with the extension replaced by `.c2pa`.
For example:

```text
figure.png -> figure.c2pa
report.pdf -> report.c2pa
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

Expected local verification shape:

```text
Manifest valid:                  yes
Claim signature valid:           yes
Signer trusted:                  no   (Local Stencila Dev (untrusted); local trust not configured)
Stencila provenance attested:    yes
Stencila reproducibility checked: not checked
```

`Signer trusted: no` is expected for `credentials init` certificates.

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

## Useful References

- C2PA specification: <https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html>
- CAI Rust SDK documentation: <https://opensource.contentauthenticity.org/docs/rust-sdk/>
- CAI manifest overview: <https://opensource.contentauthenticity.org/docs/manifest/understanding-manifest/>
- Content Credentials Verify: <https://contentcredentials.org/verify>
