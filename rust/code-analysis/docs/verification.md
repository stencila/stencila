# Extraction verification — 2026-10-11

The extracted crate passed native contract tests, existing graph snapshots,
and a linked release wasm/native comparison. Rules moved byte-for-byte: all
31 YAMLs match the pre-extraction graph rules.

## Checks

- `make -C rust/code-analysis fix test`: native Clippy fixes, formatting, all
  analyzer and graph unit/integration/doc tests; existing graph, projection,
  and diagnostic snapshots unchanged.
- Wasm Clippy for the production library, including its allocator module.
- `cargo check --locked -p stencila-code-analysis --target wasm32-unknown-unknown`.
- `make -C rust/code-analysis wasm`: linked release cdylib, all 31 fixture
  source units (all nine languages), three repeated 5,000-line Python inputs,
  and exact native/wasm equality for facts, diagnostics, and analyzer identity.
- C ABI checks: null free, zero allocations, multiplication/layout overflow,
  zero initialization and alignment, failed-resize preservation, growing to
  8 MiB, shrinking, resizing from null, and resizing to zero. All five global
  allocator/abort symbols are present. The only wasm import is wasm-bindgen's
  string conversion; there are no host libc or WASI imports.
- Dependency tree: no codec, schema, Tokio, reqwest, clap, or wasm-bindgen
  dependencies in the production library's normal/build closure.
- Standalone smoke harness native Clippy, Rust formatting, JavaScript/shell
  syntax, environment script parsing, and workflow YAML parsing.

LLVM recognizes libc allocation names and can optimize literal edge-case probes
into misleading results. The ABI test calls through opaque function pointers
so it exercises the actual linked implementations; direct wasm export calls
confirmed overflow behavior independently. No production allocator change was
needed for this test issue.

## Linked module

Rust 1.95.0, Clang 18.1.3, LLVM ar 18, wasm-bindgen 0.2.121, Node 26.4.0.
Release smoke profile: size optimization, full LTO, one codegen unit, aborting
panics, stripped symbols; no wasm-opt. Gzip measured by Node at level 9.

| Measurement | Value |
| --- | --- |
| Raw bytes | 13,086,299 |
| Gzip bytes | 1,760,610 |
| SHA-256 | `f54a22b02d6dcf2c64f6ea99e4ae65c6bf4bd89ae2fcba9b661d7b9cc3bf3599` |

These measurements include test-only bindings and allocator probes. A consumer
pins its own toolchain, remaps paths, locks dependencies, and hashes its final
module. The analyzer identity covers semantics and grammar pins separately.

## CLI comparison

A preserved pre-extraction CLI was compared with the rebuilt CLI over
`rust/graph/tests/fixtures`, using `--explain --no-c2pa --no-git-authors`.
DOT and diagnostic stderr were byte-identical. JSON was equal after excluding
volatile commit/worktree status, creation/modification dates, and start/end
times, consistent with the existing snapshot redactions.

The repository's full workspace cannot currently export a graph before or
after extraction. Both runs exit 1 with the same conflicting embedded node IDs:

- `node:rust/codec-rnw/tests/examples/chunk.rnw#cdc_chunkname`
- `node:rust/node-structuring/tests/examples/tables-with-captions-before.md#tbl_tab-s1`

Graph construction fails before JSON or DOT rendering, so successful
full-workspace export parity cannot be claimed. This pre-existing fixture
conflict is outside the extraction. The static analyzer changes do not address
it. No tracer files were changed.
