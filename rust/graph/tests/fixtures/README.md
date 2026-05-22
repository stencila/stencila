# Graph fixture plan

These directories are workspace graph snapshot fixtures. Keep
fixture notes in this file rather than inside each fixture directory, because
files inside a fixture are graph input and will appear in snapshots.

The fixture runner skips empty fixture directories. When content is added, keep
the files small and deterministic, and prefer asserting one clear graph surface
per fixture. Document-focused fixtures run with strict decode so broken fixture
documents fail loudly. Negative permissive behavior belongs in the negative
fixture, while strict option-specific failures should stay covered by dedicated
unit tests. Symlink fixtures are Unix-only in the snapshot runner.

## workspace-nested-paths-ecology

Purpose: exercise workspace filesystem graph generation for a realistic ecology
analysis repository.

The fixture is intended to contain nested authored directories such as
`data/raw`, `data/derived`, `analysis`, `figures`, and `docs`. It should include
path names with delimiters or spaces so snapshots cover workspace-relative path
normalization and graph-id escaping.

Expected graph surface:

- `Directory` and `File` nodes for workspace entries
- root and nested `PartOf` edges
- stable escaped ids for names containing `:`, `#`, `%`, or spaces
- skipped cache/build directories, if present, should not appear in the graph

## code-polyglot-tools-io

Purpose: exercise static IO extraction from JavaScript, TypeScript, and Rust
source files.

The fixture is intended to look like a scientific tooling repository with small
helper programs that read common input files and write tool-specific outputs.
Include at least one nested Rust source file so relative path resolution is
covered.

Expected graph surface:

- source-code `SoftwareSourceCode` nodes for JavaScript, TypeScript, and Rust files
- package import nodes and `ImportedBy` edges
- static local file `ReadBy` edges
- static output `Generated` edges
- `DerivedInto` edges connecting resolved inputs to resolved outputs
- correct resolution of resource literals relative to the source file path

## code-workflow-snakemake-rnaseq

Purpose: exercise Snakemake workflow graph extraction with a realistic sequencing
pipeline shape.

The fixture is intended to contain a `Snakefile` with rules such as `download`,
`trim_reads`, `align`, `counts`, and `report`, plus rule scripts under a workflow
scripts directory.

Expected graph surface:

- workflow rule nodes for each Snakemake rule
- `Generated` edges from the `Snakefile` code unit to rule nodes
- rule-level `ReadBy` edges for declared inputs
- rule-level `Generated` edges for declared outputs
- `UsedBy` edges for external rule scripts
- `DerivedInto` edges from rule inputs to rule outputs
- no cross-contamination of inputs and outputs between separate rules

## code-workflow-nextflow-metagenomics

Purpose: exercise Nextflow workflow graph extraction with a realistic
metagenomics pipeline shape.

The fixture is intended to contain `main.nf`, `nextflow.config`, sample
metadata, and representative outputs for processes such as `qc`, `classify`, and
`summarize`.

Expected graph surface:

- workflow rule nodes for recognized Nextflow `process` declarations
- `Generated` edges from the Nextflow code unit to process nodes
- process-level `ReadBy` edges for static `path` inputs
- process-level `Generated` edges for static `path` outputs
- `CalledBy` edges for executable blocks such as `script`, `shell`, or `exec`
- `DerivedInto` edges from process inputs to process outputs
- dynamic channel expressions should not be mistaken for concrete resources

## code-python-r-dataframe-provenance

Purpose: exercise static dataflow extraction from Python and R analysis scripts.

The fixture is intended to represent a small data analysis project that reads
raw tabular data, derives cleaned/model outputs, and writes figures or summary
tables. Python and R scripts should use common scientific packages and dataframe
column access patterns.

Expected graph surface:

- source-code `SoftwareSourceCode` nodes for Python and R files
- imported package `SoftwareApplication` nodes
- variable and function nodes for assignments, declarations, uses, and calls
- `ImportedBy`, `ReadBy`, `Generated`, `UsedBy`, and `CalledBy` edges
- `DatatableColumn` nodes for recognized dataframe column accesses
- `DerivedInto` edges from inputs and columns to generated outputs

## document-executable-reactivity-notebook

Purpose: exercise executable document graph extraction and document-local
reactivity.

The fixture is intended to contain a notebook-like Stencila document with setup
chunks, downstream analysis chunks, inline code expressions, local file reads,
and recorded outputs.

Expected graph surface:

- `CodeChunk` and `CodeExpression` boundary nodes
- static code facts extracted from executable document nodes
- `DependsOn` edges between chunks based on symbol use and prior definitions
- local file `ReadBy` edges resolved relative to the document file
- recorded output nodes and `Generated` edges with execute-action metadata when executed outputs are present
- output nodes should remain shallow enough for stable snapshots

## document-report-references-citations

Purpose: exercise decoded document graph extraction for manuscript-style
references and structure.

The fixture is intended to contain a Stencila Markdown manuscript with headings,
internal links, external links, citations, figure/media references, and include
blocks that point to local data or table files.

Expected graph surface:

- decoded document boundary nodes such as `Article`, `Heading`, `Link`, `Citation`, `Reference`, `ImageObject`, and `IncludeBlock`
- `ConvertedInto` edges from source document files to decoded document roots
- document structural `PartOf` edges
- local media and link `ReferencedBy` edges
- citation `CitedBy` edges
- include `TranscludedBy` edges
- external resources represented as resource nodes when appropriate

## environment-manifests-lockfiles-polyglot

Purpose: exercise declared computational environment extraction across the
supported manifest ecosystems.

The fixture is intended to look like a reproducible scientific software
workspace with Python, Node.js, Rust, R, and Julia project metadata. It should
include manifests and sibling lockfiles where possible.

Expected graph surface:

- environment `SoftwareApplication` nodes for supported manifests
- package `SoftwareSourceCode` nodes using package-url identifiers
- manifest `DerivedInto` edges to environment nodes
- package `PartOf` edges to environment nodes with declared and static-analysis evidence
- lockfile `ReferencedBy` edges to associated environment nodes
- SHA-256 identifiers on manifest and lockfile `File` nodes

## workspace-negative-dynamic-invalid

Purpose: exercise best-effort workspace graph behavior for realistic messy
repository contents.

The fixture is intended to contain unsupported source files such as SAS,
malformed supported code, dynamic path expressions, missing local references,
and an invalid environment manifest. It should be used to assert absence of
false-positive graph edges as much as presence of inventory nodes.

Expected graph surface:

- ordinary workspace `Directory` and `File` nodes should still be emitted
- unsupported source languages should not produce code-unit nodes
- syntax-error source files may produce code-unit nodes but should not emit partial static facts
- dynamic resource expressions should not create concrete `ReadBy`, `Generated`, or synthetic resource edges
- invalid manifests should be ignored in permissive mode and rejected only by strict option-specific tests
- missing local references should not create dangling edges

## workspace-symlinks-publication-assets

Purpose: exercise symbolic-link handling in a realistic publication asset
workspace.

The fixture is intended to model a manuscript repository where public-facing
asset paths point at versioned source files, for example a symlink from
`docs/figures/latest.png` to `figures/model-v2.png`.

Expected graph surface:

- `SymbolicLink` nodes for link entries
- `PartOf` edges from symlinks to their containing directories
- `ReferencedBy` edges from in-workspace symlink targets to symlink nodes
- no document decoding or code analysis through symlink targets
- unresolved or outside-workspace symlink targets should not create dangling edges
