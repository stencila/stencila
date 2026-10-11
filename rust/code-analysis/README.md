# Static code analysis

`stencila-code-analysis` analyzes source text without executing it or accessing
the filesystem, network, or an async runtime. It is shared by the CLI graph
collectors and WebAssembly consumers. It has no codecs, schema, or wasm-bindgen
dependency. Document decoding, filesystem inventory, environment manifests,
cross-unit graph relationships, and schema conversion belong to `stencila-graph`.

```rust
use stencila_code_analysis::{analyze_file, CodeLanguage, ANALYZER_IDENTITY};

let language = CodeLanguage::from_path("logical/analysis.py")
    .ok_or("unsupported language")?;
let result = analyze_file(language, "logical/analysis.py", "open('data.csv')\n");
// Serialize result and ANALYZER_IDENTITY separately when recording findings.
# Ok::<(), Box<dyn std::error::Error>>(())
```

Supported languages are Python, R, JavaScript, TypeScript, Julia, MATLAB, Rust,
Snakemake, and Nextflow. Nextflow uses the existing text extractor; the other
eight use pinned tree-sitter grammars and embedded ast-grep rules. Bash remains
unsupported (`None` from language detection). Detection accepts logical paths
with either slash convention, extensions (also via `from_extension`), `Snakefile`,
and `nextflow.config`. It never calls `Path::is_dir`.

Resolution is confined to a single source unit: literals, module constants,
single-assignment locals, and arguments to unambiguous local functions. It does
not read or resolve constants imported from sibling files, so there is no
set-of-files API. Environment manifest parsers could later take strings here;
manifest discovery and reading would remain in the graph crate.

## JSON contract

`FORMAT_VERSION` is **1**. `Analysis` serializes as an object with `formatVersion`,
`language`, `path`, `facts`, and `diagnostics`. The path is preserved verbatim.
Fields use camelCase, including nested facts and diagnostics. Optional values
are explicit `null`; empty arrays and objects are retained. Sets serialize as
sorted arrays; maps have sorted string keys. Offsets and function `start`/`end`
ranges are UTF-8 byte offsets into the supplied source (end exclusive).

The language strings are `python`, `r`, `javascript`, `typescript`, `julia`,
`matlab`, `rust`, `snakemake`, and `nextflow`. Deserialization also accepts the
historical language aliases used by rule YAMLs.

The `facts` object contains:

| Field | Representation |
| --- | --- |
| `imports` | Array of `{ecosystem, name}` |
| `assignments`, `declarations`, `uses`, `calls` | Arrays of symbol strings |
| `functionDeclarations` | Array of `{name, start, end}` |
| `io` | Array of I/O objects described below |
| `columns` | Array of `{dataframe, column, source}` |
| `variableFlows` | Array of `{target, source, targetOffset}` |
| `workflowUnits`, `scriptLinks` | Arrays of name/path strings |
| `scriptExecutions` | Array of `{path, kind, offset, command}` |
| `workflowUnitFacts` | Map from unit name to `{io, scriptLinks, scriptExecutions, calls}` |
| `syntaxError` | Boolean; true means parsing failed and other facts are empty |

I/O objects have `direction`, `path`, `operationOffset`, `target`, `targetOffset`,
`value`, `valueOffset`, `function`, `mode`, and `unresolvedReason`. Directions
are `read`, `write`, or `readWrite`; modes are `read`, `write`, `append`, or
`readWrite`. Paths are adjacent-tagged objects:

- `{"kind":"static","value":"data.csv"}`
- `{"kind":"unknown","value":"path_expression"}`
- `{"kind":"template","value":{"expression":"...","segments":["data/"],"unresolved":["name"]}}`

Script execution kinds are `script` or `shell`. Unresolved reasons are objects
tagged by `kind`: `notALiteral`, `partialTemplate`, `unboundIdentifier`,
`multipleAssignments`, `conditionalAssignment`, `useBeforeDefinition`,
`nonLiteralValue`, `unresolvedParameter`, or `ambiguousCallee`. Identifier
reasons carry `name`; `unresolvedParameter` carries `parameter` and `function`;
`ambiguousCallee` carries `callee`.

Diagnostics are internally tagged by `kind`. A syntax failure is
`{"kind":"syntaxError"}` with no fabricated span. `unresolvedIo` diagnostics
carry `direction`, `expression`, `offset`, `location`, `function`, `mode`,
`knownSegments`, `workflowUnit`, `reason`, `message`, and `remedy`. Their location
is either null or `{start: {line, column, offset}, end: null}`: the extractor
knows the operation's start, not its full extent. Lines and Unicode scalar
columns are zero-based; offsets are UTF-8 bytes. Invalid offsets have no
location. Workflow-unit names are null for whole-file operations.

Private import aliases, variable-source maps, definition/use offsets, and
read-before-write bookkeeping are omitted. Read-only methods on live `CodeFacts`
serve graph integration. Deserialization recreates public facts with empty
bookkeeping; it is for storage/interchange, not reconstructing document
reactivity metadata.

Changing these fields or enum representations requires a new format version.
Every fixture source unit has a JSON snapshot in `tests/snapshots`.

## Analyzer identity

`ANALYZER_IDENTITY` serializes `crateVersion`, `sourceDigest`, and `grammars`.
The digest is lowercase SHA-256 over sorted relative paths and file contents
under `src/`, `rules/`, and `include/`, plus `Cargo.toml` and `build.rs`. Each
path and contents are preceded by their u64 little-endian byte lengths. Host
paths and timestamps are excluded. Grammar records have `package`, `version`,
and `gitCommit` (null for registry packages). Dependencies are exact-pinned;
Snakemake 2.0.0 is pinned at
`7731408e5e8095fe242fdd423c3d3ae886fbf9fd`. Consumers also retain their Cargo.lock
and hash the final module; this identity describes analyzer semantics, not the
entire build toolchain or a module's bytes.

## Bare WebAssembly builds

The supported target is `wasm32-unknown-unknown`. The tested and supported C
compiler is **Clang 18.1.3**, with LLVM ar 18. The spike was tested with Rust
1.95.0, Node 26.4.0, and wasm-bindgen 0.2.121. Native builds need no compatibility
flags. The crate has no bindings; the smoke harness under `tests/wasm` supplies
test-only bindings.

Install `clang-18` and `llvm-18`, install the Rust target, resolve and lock your
consumer's dependencies, then derive the exact environment:

```sh
rustup target add wasm32-unknown-unknown
eval "$(python3 rust/code-analysis/scripts/wasm-env.py)"
cargo check --locked -p stencila-code-analysis --target wasm32-unknown-unknown
```

For another checkout or consumer, pass `--manifest-path /path/to/Cargo.toml`.
`--offline` is available after fetching dependencies. The script uses Cargo
metadata to find the pinned `tree-sitter-language` package's `wasm/include`
directory and this crate's own `include/compat.h`, validates the compiler, and
prints shell-quoted `CC_wasm32_unknown_unknown`, `AR_wasm32_unknown_unknown`, and
`CFLAGS_wasm32_unknown_unknown`. Target-specific compiler overrides are accepted.
Existing target CFLAGS are appended, so consumers can add their own prefix maps
and reproducible-build options. The effective C flags are:

```text
-I <tree-sitter-language-0.1.7>/wasm/include -include <code-analysis>/include/compat.h
```

The compatibility header includes boolean definitions for R, a static inline
`strcmp` for MATLAB, and trapping assertions that avoid duplicate upstream
`__assert_fail` definitions. It does not embed `__FILE__`, host paths, or host
imports. Consumers must pin the compiler and remap their checkout/registry
paths in C and Rust flags (for example `-ffile-prefix-map`/`-fdebug-prefix-map`
and `--remap-path-prefix`). The environment script does not prescribe those
consumer-specific maps.

### Allocator and linking policy

On bare wasm32 only, this crate defines the **global C ABI symbols `malloc`,
`calloc`, `realloc`, `free`, and `abort`**. The allocation functions use Rust's
global allocator with a 16-byte-aligned size prefix; `abort` traps. Native and
WASI builds do not compile this replacement. Rust's bare wasm default allocator
does not call these C symbols. A custom Rust global allocator must likewise
avoid calling them recursively.

All five symbols must be linked together. Their definitions prevent the linker
from pulling in `tree-sitter-language`'s scanner-only `stdlib.c` bump allocator,
which otherwise starts without heap initialization and has a 4 MiB limit. The
remaining upstream stdio/string shims can still be linked. Consumers must not
provide competing definitions or link that standalone scanner allocator (for
example by calling its `reset_heap`). C pointers must be allocated, resized,
and freed by the same allocator; pointers from Rust containers must never be
passed to C `free`. Build a linked module and execute it: `cargo check` cannot
detect duplicate symbols or broken C allocation.

## Verification

`make -C rust/code-analysis test` runs native crate and graph tests.
`make -C rust/code-analysis wasm` runs a release linked cdylib and native parity
check, including all 31 source fixtures, every supported language, and a repeated
5,000-line allocator stress case. The CI workflow uses the same commands.
