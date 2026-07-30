---
name: conversion
description: Converting documents between formats with stencila convert - lossless vs lossy targets, quantifying conversion losses, decoding options, and ingesting documents by DOI, arXiv, or PubMed Central identifier. Use when converting to or from DOCX, PDF, LaTeX, JATS, IPYNB, MyST, Quarto or other formats, or importing published papers.
user-invocable: false
---

# Converting Stencila documents

```sh
NO_COLOR=1 stencila convert input.docx output.smd --yes
```

Formats are inferred from file extensions; `--from`/`--to` override. The
input may be a path, a URL, or a bare identifier (see Ingestion below).
`stencila formats list` prints the live capability table — which formats
decode (`From`), encode (`To`), and are lossless. Consult it rather than
guessing.

## Lossless vs lossy

Only the data formats round-trip losslessly: **CBOR, CBOR+Zstd, JSON,
JSON+Zip, JSON5, JSON-LD, YAML**. Everything else — DOCX, PDF, LaTeX, JATS,
HTML, Markdown flavours, IPYNB — is lossy to some degree.

When converting to a lossy target, quantify what is dropped:

- `--input-losses` — action on losses decoding the input
- `--output-losses` — action on losses encoding the output

Both accept `ignore`, `trace`, `debug` (default), `info`, `warn`, `error`,
`abort`, or a `.json`/`.yaml` filename to write the losses to.
After a lossy conversion, tell the user what was lost rather than silently
producing a smaller document.

## Decoding options that matter

- `--fine` / `--coarse` — decoding granularity. Fine (the default for most
  formats) decodes to the finest structure; coarse preserves un-modelled
  markup in larger blocks and is the default for LaTeX. Use `--coarse` when
  fine decoding of a partially supported format mangles structure.
- `--pages N-M` / `--exclude-pages` — page selection for multi-page inputs
  (e.g. PDFs): `--pages 1,3,5-7`, `--pages 2-`, keywords `odd`/`even`.
- `--include-structuring` / `--exclude-structuring` — structuring operations
  that infer document structure (sections, abstracts, references) from
  loosely structured input. Example:
  `--include-structuring sections-to-abstract`.
- `--ignore-artifacts` — re-download / re-process instead of using cached
  intermediate artifacts in `.stencila/artifacts/`.

## Ingesting published papers

`convert` accepts identifiers directly as the input — a DOI, arXiv id, or
PubMed Central id — and fetches the document:

```sh
NO_COLOR=1 stencila convert 10.1371/journal.pcbi.1011999 paper.smd --yes
NO_COLOR=1 stencila convert arxiv:2004.10643 paper.smd --yes
NO_COLOR=1 stencila convert PMC7067710 paper.smd --yes
```

DOIs may be bare, `doi:`-prefixed, or `https://doi.org/…` URLs; arXiv ids
may be `arXiv:`-prefixed or arxiv.org URLs. OpenAlex work ids (e.g.
`W2741809807`) are also accepted.

## External tools

Some formats encode via external tools. `--tool pandoc` selects Pandoc for
encoding, `--from-tool` for decoding, and arguments after `--` pass through
to the tool:

```sh
NO_COLOR=1 stencila convert doc.smd doc.docx --tool pandoc --yes -- --reference-doc=style.docx
```

## Checking a conversion

Round-trip through JSON to see exactly what Stencila modelled:

```sh
NO_COLOR=1 stencila convert doc.docx --to json --yes | head -100
```
