# WebAssembly code analysis feasibility spike (phase 1 archive)

Go for extraction into `stencila-code-analysis`, with the compatibility layer described below. All eight tree-sitter grammars, ast-grep-core, ast-grep-config and stencila-format build for `wasm32-unknown-unknown`. Native and wasm facts match for every supported source unit in the graph fixtures. Representative and 5,000-line inputs run for all nine supported languages, including Nextflow's text analyzer.

This is the historical phase 1 report, reviewed before extraction. The spike was committed as `cc3ac43282` and subsequently superseded by this crate. Its original harness and lockfile remain in that commit. See [the crate README](../README.md) for the supported build environment and current API. At that point the spike lived in a separate worktree on `code-analysis-crate`, based on `main` at `63a74527fd`. Production graph and tracer files were unchanged during phase 1.

## Build results and required compatibility

The default build fails for every grammar with missing `stdlib.h`. Tree-sitter 0.26.9 obtains headers and C libc implementations from tree-sitter-language 0.1.7, but the grammar build scripts do not automatically consume that header path. Passing it through target-specific CFLAGS makes the grammars build, subject to the fixes below. No grammar is excluded.

| Grammar | Version | Additional compatibility beyond the shared header path |
| --- | --- | --- |
| Python | 0.25.0 | Trapping assertion macro |
| R | 1.2.0 | Include stdbool before wctype |
| JavaScript | 0.25.0 | Common headers and allocator |
| TypeScript | 0.23.2 | Common headers and allocator |
| Julia | 0.23.1 | Common headers and allocator |
| MATLAB | 1.3.0 | Inline strcmp |
| Rust | 0.24.2 | Common headers and allocator |
| Snakemake | 2.0.0 | Trapping assertion macro |

Snakemake is pinned to `v2.0.0`, commit `7731408e5e8095fe242fdd423c3d3ae886fbf9fd`. Both ast-grep crates are 0.42.3. Dependency resolutions are recorded in `Cargo.lock` in the spike commit.

[include/compat.h](../include/compat.h) supplies `stdbool.h` before R's scanner includes `wctype.h`, implements the `strcmp` missing from MATLAB's string shim, and replaces the upstream assertion header with a trapping macro. The upstream header defines external `__assert_fail` in each translation unit; linking Python, Snakemake and the core runtime together otherwise produces duplicate symbols. Assertions remain enabled.

The upstream `stdlib.c` allocator is unsuitable for the combined Rust module. It starts with a null heap pointer until an external host calls `reset_heap`, and caps its heap at 4 MiB. The initial build passed fixture comparisons but crashed on the 5,000-line Python input with an out-of-bounds memory access. [src/wasm_allocator.rs](../src/wasm_allocator.rs) supplies `malloc`, `calloc`, `realloc`, `free` and trapping `abort` together, preventing that archive member from being linked. C allocations use Rust's allocator with a 16-byte aligned prefix recording the allocation layout. All large inputs then pass, including repeated analysis in the same instance.

The allocator exports are global C ABI symbols. The extracted crate or a patched dependency must own this compatibility code; a consumer that supplies another libc allocator needs a deliberate linking policy. A cargo check alone cannot catch the original linking and runtime failures, so preserving a linked wasm smoke/parity check is advisable alongside the required CI check.

## Toolchain and build flags

Tested on Linux x86_64 with Rust 1.95.0, Ubuntu Clang 18.1.3, llvm-ar 18, wasm-bindgen and its CLI 0.2.121, Python 3.12.3 and Node 26.4.0. The CPU is an Intel Core i7-13700H. These are tested versions, not established minimum versions.

The required environment is:

```sh
export CC_wasm32_unknown_unknown=clang
export AR_wasm32_unknown_unknown=llvm-ar
export CFLAGS_wasm32_unknown_unknown="-I${tree_sitter_language_dir}/wasm/include -include ${spike_dir}/include/compat.h"
```

`run.py` obtains `tree_sitter_language_dir` from Cargo metadata and resolves the spike directory automatically. cc adds `--target=wasm32-unknown-unknown`, section-splitting flags and the grammar's own C-standard flags. Release C compilation uses `-Oz`; the Rust release profile uses `opt-level = "z"`, LTO, one codegen unit, aborting panics and stripped symbols. No WASI sysroot, container or external libc is needed. Julia's generated parser is approximately 53 MiB of C and takes several minutes to compile in some profiles.

The corrected unknown-unknown target succeeds, so WASI fallback is unnecessary. The final module has **zero WASI imports and zero host libc imports**. Its only import is wasm-bindgen's string-to-JavaScript conversion used to return unsupported-language errors. [wasm-spike-results.json](wasm-spike-results.json) records the exact import name. Bash remains unsupported in native and wasm smoke checks.

## Module size

The full module is **13,011,306 bytes raw** and **1,672,017 bytes gzipped** (13.01 MB and 1.67 MB in decimal units). This fits the stated 10 MB compressed module budget. These sizes cover the wasm module; the lab's bindings and other Worker code use additional budget. Gzip uses Python gzip with zlib 1.3, level 9 and mtime zero; no wasm-opt pass is applied.

Each row removes one grammar adapter, relinks the complete analyzer, and measures the resulting module. The decreases are marginal contributions, not standalone grammar-module sizes; shared code, LTO and compression make them non-additive. TypeScript includes its TypeScript grammar; unused TSX is removed by linking. Nextflow contributes text analysis and has no tree-sitter grammar.

| Grammar omitted | Remaining raw bytes | Remaining gzip bytes | Raw decrease | Gzip decrease |
| --- | ---: | ---: | ---: | ---: |
| Python | 12,562,246 | 1,608,981 | 449,060 | 63,036 |
| R | 12,534,941 | 1,626,866 | 476,365 | 45,151 |
| JavaScript | 12,609,738 | 1,625,311 | 401,568 | 46,706 |
| TypeScript | 11,610,100 | 1,541,799 | 1,401,206 | 130,218 |
| Julia | 6,747,789 | 1,106,232 | 6,263,517 | 565,785 |
| MATLAB | 12,594,985 | 1,611,641 | 416,321 | 60,376 |
| Rust | 11,906,363 | 1,559,082 | 1,104,943 | 112,935 |
| Snakemake | 12,214,114 | 1,585,658 | 797,192 | 86,359 |

[wasm-spike-sizes.json](wasm-spike-sizes.json) records the matrix. The full module SHA-256 is `0a94a80c0b0e07c8c744b57b492c9cc4ce43f254623602ff45290cd4d31a305c`.

## Startup time

Twenty fresh Node processes instantiate the complete module. The host error-conversion import is stubbed during startup measurement; it is never called during instantiation. V8 can defer function compilation, so the first analysis also measures compilation triggered by that language.

| Operation | Median ms | Minimum ms | Maximum ms |
| --- | ---: | ---: | ---: |
| WebAssembly.Module compilation | 6.48 | 5.93 | 10.98 |
| WebAssembly.Instance instantiation | 4.16 | 3.71 | 5.53 |
| Read module plus compile and instantiate | 15.74 | 14.14 | 20.75 |

Node process startup is excluded. These are local Node measurements, not Worker cold-start predictions.

## Analysis time and peak memory

Each input runs in a fresh Node process with a fresh wasm instance. Cold time is the first wrapper call; warm time is the median of five further calls. Calls include source transfer, analysis, serialization of every fact field and transfer of the resulting JSON string. JavaScript JSON parsing is excluded. Each repeated result must be byte-identical to the first result.

Peak wasm memory is the capacity high-water mark of linear memory after all six calls, including static grammar data, the stack and both Rust and C allocations. Memory grows monotonically, making this observable without allocator sampling; it is not peak live heap. Peak RSS is the whole Node process maximum, including V8, wasm compilation, JavaScript and the input corpus. MiB uses 1,048,576 bytes.

Representative inputs come from the polyglot fixture, Snakefile and the Nextflow workflow.

| Language | Lines | Cold ms | Warm ms | Peak wasm MiB | Peak RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| Python | 9 | 13.16 | 0.99 | 11.94 | 103.5 |
| R | 4 | 12.76 | 0.94 | 11.94 | 102.2 |
| JavaScript | 8 | 17.51 | 1.29 | 11.94 | 104.2 |
| TypeScript | 6 | 13.04 | 1.09 | 11.94 | 107.5 |
| Julia | 4 | 40.23 | 13.01 | 12.19 | 222.1 |
| MATLAB | 5 | 24.32 | 3.86 | 12.19 | 108.2 |
| Rust | 13 | 18.24 | 3.57 | 12.00 | 105.3 |
| Snakemake | 23 | 11.02 | 1.01 | 11.94 | 105.8 |
| Nextflow | 30 | 4.07 | 0.12 | 11.88 | 94.1 |

Large inputs have exactly 5,000 lines. General-purpose languages have distinct assignments and a file read every 25 lines. Snakemake has 1,000 rules with inputs, outputs and shell commands; Nextflow has 1,000 processes. All return `syntax_error: false`. These exercise large static inputs, rather than representing every possible analysis workload.

| Language | Cold ms | Warm ms | Peak wasm MiB | Peak RSS MiB |
| --- | ---: | ---: | ---: | ---: |
| Python | 156.91 | 104.48 | 19.31 | 117.7 |
| R | 158.95 | 102.52 | 18.38 | 116.9 |
| JavaScript | 170.27 | 102.92 | 19.56 | 119.6 |
| TypeScript | 186.91 | 125.71 | 21.81 | 123.3 |
| Julia | 685.03 | 536.71 | 18.88 | 232.0 |
| MATLAB | 178.31 | 117.39 | 17.25 | 119.0 |
| Rust | 193.46 | 129.82 | 18.00 | 115.6 |
| Snakemake | 129.61 | 72.17 | 22.81 | 119.4 |
| Nextflow | 21.20 | 9.20 | 14.56 | 105.6 |

Julia has a much higher Node RSS peak than its linear-memory footprint. The lab should validate this language in the actual Worker runtime before deployment: Node process RSS and Worker isolate accounting are different measurements. The measurements establish buildability and Node execution, with production Worker behavior still a lab-side question.

## Native parity and verification

**31 of 31 supported fixture source units match**, including standalone scripts, `nextflow.config`, embedded JSON notebook chunks and expressions, executable Markdown fences, and malformed-source fixtures. The comparison recursively checks every `CodeFacts` field, including private offsets and resolution bookkeeping, workflow facts and unresolved I/O reasons. Source paths and per-result hashes are recorded in `results.json`.

The native runner compiles copies of the existing graph analysis passes. `prepare.py` changes only the Format import, adds serde serialization derives, and selects the pure parts of package, reference and diagnostic types. It copies all rule YAMLs unchanged. No extraction or normalization rule is rewritten. Its temporary JSON representation includes internal fields for exhaustive parity; it is not the proposed public JSON contract.

Passing checks:

- Native Clippy with fixes and all targets, and wasm Clippy with fixes and all targets.
- `cargo fmt --check` and two native smoke tests.
- `cargo check` for `wasm32-unknown-unknown`, a release cdylib build, and wasm-bindgen processing.
- Native versus wasm parity for 31 sources, plus 18 benchmarks with repeated-result checks.
- Dependency-tree inspection: no stencila-codecs, stencila-schema, Tokio, reqwest or clap.
- Python compilation and JavaScript syntax checks for the spike harness.

## Extraction boundary findings

`analyze_source` and its resolution pass operate on one source unit. Resolution includes local/module constants and local function arguments, but does not load sibling modules or resolve an imported constant from another file. Workspace graph construction and document reactivity connect facts later; they are integration behavior, not a cross-file analyzer. A set-of-files API is therefore not required to preserve today's core resolution.

`stencila-format` is lightweight enough to build for wasm. However, its `Format::from_path` calls `Path::is_dir`, so the new crate must detect logical paths without that method. Pure `Format::from_name` plus filename exceptions, or a detector on `CodeLanguage`, can preserve supported extensions and special names such as Snakefile and nextflow.config. The public selector can remain `CodeLanguage`.

The pure unresolved-I/O reasons already live beside the facts. Schema-based diagnostic locations, document identifiers and graph projections stay in stencila-graph; phase 2 can expose plain spans and convert them at that boundary. Its public JSON must omit private bookkeeping and explicitly document/version the chosen field and enum representation. Analyzer identity should cover the rule/analysis sources and pinned grammar versions in addition to the crate version.

Environment-manifest parsing could later accept logical paths and manifest text without filesystem access. Manifest reads and schema/package-node projection would remain in graph; adding that API is outside this extraction.

## Reproduce the historical spike

Check out commit `cc3ac43282` in a scratch directory to restore the original spike. Install Clang and llvm-ar on PATH, the Rust target and the matching wasm-bindgen CLI. Populate the locked dependency cache once, then run:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.121 --locked
cargo fetch --manifest-path rust/graph/wasm-spike/Cargo.toml --locked
python3 rust/graph/wasm-spike/run.py --matrix
```

`--wasm-bindgen /path/to/wasm-bindgen` selects an existing CLI. Omit `--matrix` for a full-module build and parity/benchmark run without grammar omission builds. The script uses offline, locked Cargo builds and does not change production source files. It creates ignored source copies and generated bindings, and updates the JSON measurements. Node must be allowed to launch child processes to measure startup and RSS in fresh processes.

For a check without rebuilding the size matrix, use the C environment above with:

```sh
cargo check --manifest-path rust/graph/wasm-spike/Cargo.toml --target wasm32-unknown-unknown --locked --offline
```

The phase 2 crate will use the same C setup and allocator compatibility, while the lab supplies its own bindings. Existing graph snapshots and CLI JSON, DOT and explain output remain the integration guards for that phase.
