# JATS media files and CLI conversion

Path decoding accepts `.jats.xml`, `.xml`, and `.nxml` inputs containing a JATS
article. Graphics, inline graphics, media, and supplement targets are resolved
relative to the XML file. NIH manuscript figures in a trailing `floats-group`
(or older `floats-wrap`) are retained too. Decoding a string does not assume
that its hrefs refer to the current working directory.

`JatsCodec::from_path` does not embed media or supplements unless
`DecodeOptions::embed_media` or `DecodeOptions::embed_supplements` is set, so
by default they remain references to the files.

## Reference and extraction policy

CLI conversion of a file keeps media and supplements as file references, unless
`--embed-media` or `--embed-supplements` is passed, when every output is in a
format that keeps references: JSON, JSON5, JSON-LD, YAML, CBOR, Markdown
flavours, HTML and LaTeX (`stencila_codecs::keeps_media_references`).
References are relative to each output document, including when several outputs
are requested. String/stdout output uses the current working directory as the
reference base.

Relocation is opt-in (`EncodeOptions::relocate_media`), which the CLI sets for
its outputs, and is done by the `codecs` dispatcher so that it applies to every
format that keeps references. Other serialization (e.g. the document cache)
preserves references unchanged. Bundling formats (e.g. DOCX, PDF) are not
relocated, because their encoders need references that resolve when encoding,
and the CLI keeps the library defaults (embedding) for archives converted to
them. Relocation is also skipped when collecting media (e.g. for Sites), which
resolves references itself. Relative references are resolved from the
symlink-resolved directory of the input, matching the JATS decoder, and include
media in nested works such as sub-articles and graphical abstracts.

`--extract-media DIR` extracts data URIs and copies local originals into DIR;
both operations preserve bytes and do not resize images. Copied names include a
content hash to avoid collisions. Original files are never modified.
`--extract-supplements DIR` also copies original supplement files; already
embedded supplemental works retain the existing CLI extraction to CBOR
behaviour.

Explicit embedding still uses the existing web optimization settings (1200px
image width, TIFF converted to PNG, audio/video optimized when the relevant
tools are installed). Declared or inferred MIME types follow any conversion.

Compressed MECA and PMC inputs decoded with embedding disabled retain referenced
originals in side directories beside the input archive, so references survive
the temporary extraction directory. For `article.meca` these are
`article.meca.media` and `article.meca.supplements`: the suffix is appended to the
whole file name so that they never coincide with the default `<output>.media`
directory of an output beside the input (e.g. `article.md`), which is removed
before encoding. An unpacked MECA bundle references its existing files.
These retained originals can subsequently be collected into another directory
using the extraction flags. Side directories are not automatically cleaned. If
the archive's directory is not writable, decoding fails with an error naming it
and suggesting embedding or a writable copy.

## Href matching

An existing exact href always wins, including an explicitly selected thumbnail.
Otherwise, the resolver searches only the href's containing directory for files
whose stem equals the entire href basename. This also handles publisher stems
that contain dots. Candidates are ordered by:

1. Vector originals (SVG, EPS, PDF) before raster files.
2. Greatest raster pixel area, read from the image header without decoding or
   resizing the image.
3. Format preference: SVG, EPS, PDF, TIFF, PNG, JPEG, GIF, other formats.
4. Lexical filename order.

This prefers scalable originals and larger images without assuming that the
largest compressed file has the most detail. TIFF and PNG break resolution ties
ahead of lossy formats. This is a deterministic heuristic, not proof of which
publisher asset was the original. A full MIME type provided by JATS is retained;
when absent, it is inferred from the selected file extension. Remote URLs, data
URIs, and fragment references are preserved. Missing local hrefs remain in the
document and are reported as decode losses rather than causing decode errors.

## MECA compatibility and callers

Unlike the JATS codec, the MECA and PMC codecs embed both media and supplements
by default. MECA honors explicit `Some(false)` and `Some(true)` independently.
The `convert` command explicitly selects references by default for file inputs
when every output keeps references (see above). For example, a MECA archive
converted to Markdown links to the original files (e.g. TIFF) rather than to
resized PNGs extracted to `<output>.media`. Callers needing embedded media can
pass `--embed-media --embed-supplements`. Identifier inputs (e.g. PMC ids) and
bundling output formats keep the library defaults, because downloaded archives
are temporary and bundling encoders do not use rebased references.
Supplements already embedded during decoding are not embedded again by the
CLI's `--embed-supplements` pass.

The direct callers and relevant indirect consumers are:

- `codec-openrxiv/src/decode.rs`, `from_url`: forwards decode options to MECA;
  embedding is the default, and explicit false flags take effect.
- `codecs/src/lib.rs`: registers `MecaCodec` for generic path/identifier decoding.
- `codec-meca/tests/examples.rs`: decodes with only `reproducible: false`, so its
  snapshots exercise default embedding.
- `convert-server/src/lib.rs`, `convert_request`: uses the generic dispatcher
  with default decode embedding options.
- `convert-server/src/private.rs`, `convert_private_bundle`: extracts MECA itself,
  selects the manifest manuscript, and decodes JATS with embedding explicitly
  disabled. It does not call `MecaCodec`, and retains its own artifact policy.
  Its MECA API tests are in `convert-server/tests/api.rs`.
- The generic `codecs` entry points also serve the CLI, documents, and bindings.
  The tests in `cli/tests/convert_media.rs` and `codec-meca/tests/media.rs`
  distinguish CLI reference defaults from library embedding defaults.

## Commands and output

From the repository root, after building the CLI:

```sh
stencila convert rust/codec-jats/tests/media/PMC5754353.1/PMC5754353.1.xml /tmp/article.json
stencila convert rust/codec-jats/tests/media/extensionless/article.xml /tmp/figures.json --extract-media /tmp/originals
stencila convert rust/codec-jats/tests/media/nih-manuscript/article.xml /tmp/manuscript.json --input-losses /tmp/losses.json
stencila convert rust/codec-meca/tests/fixtures/originals.meca /tmp/bundle.json
stencila convert rust/codec-meca/tests/fixtures/originals.meca /tmp/embedded.json --embed-media --embed-supplements
```

For example, with input and output in sibling directories:

```json
{
  "type": "ImageObject",
  "contentUrl": "../PMC5754353.1/41467_2017_2484_Fig1_HTML.jpg",
  "mediaType": "image/jpeg"
}
```

The extracted version references a byte-identical copy such as
`../originals/41467_2017_2484_Fig1_HTML-bdf79530b420e5f7.jpg`.

`--input-losses losses.json` produces a machine-readable map of labels to
counts, rather than an array. YAML is also supported. For the synthetic NIH
fixture, one entry is:

```json
{
  "//article[1]/floats-group[1]/fig[1]/graphic[1]/@xlink:href (unresolved: nihms-1836442-f0001.jpg)": 1
}
```

The missing-media labels include indexed source XML paths and original hrefs,
so that each missing resource can be identified; other decode losses use
unindexed paths. `stencila_codec_jats::classify` ignores the indices and the
note in parentheses. A loss file is only written when losses are nonempty, and
its parent directory must exist.

## Real-layout checks

See [fixture provenance](tests/media/README.md) for licensing and reductions.
The full PMC5754353.1 JATS and one original figure were fetched anonymously from
`https://pmc-oa-opendata.s3.amazonaws.com/`. The original figure and its extracted
copy have the same SHA-256:
`40df94311a22720baad65cee99acc89fbbf856a3c7efeddccb0631dc62d1a9dc`.

The Europe PMC archive from
`https://www.ebi.ac.uk/europepmc/webservices/rest/PMC5754353/supplementaryFiles`
was unpacked beside that JATS. It is flat and contains the six JPG figures,
GIF thumbnails, and two PDF supplements. All six image references resolved and
there were no unresolved-media losses. Explicit JPG hrefs retain the JPGs
regardless of the additional GIF thumbnails. This layout works through the same
resolver and requires no special Europe PMC handling.

The full NIHMS1836442 manuscript (PMC9524584.1) was also fetched for a local
check. Its bucket has XML/text/JSON and no media. Its copyrighted article content
is not committed; the regression fixture reproduces only its file layout. The
full manuscript decodes successfully with four figure references and six
unresolved-media losses (four figures and two supplements).
