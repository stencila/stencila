# Tool Schema Fixtures

These JSON files are reference schemas for the core tools, derived from the
Coding Agent Loop Specification sections 3.3, 3.6, and Appendix A (spec
revision: 2025-06, `specs/coding-agent-loop-spec.md` in this repository).

Each file deserializes as a `ToolDefinition` from `stencila-models3` and serves
as a test fixture in `tests/spec_3_tools.rs` or `tests/spec_3_patch.rs`. The
schema parity tests verify that the `definition()` function of each tool module
produces a schema matching its fixture.

**Note on duplication:** The schema content is intentionally duplicated between
these fixture files and the Rust `definition()` functions in `src/tools/`. The
parity tests catch accidental divergence between the two. This tests internal
consistency rather than external contract drift — the fixtures are authored here,
not vendored from upstream provider repos. If external contract testing is needed
in the future, these fixtures should be replaced with schemas extracted from the
upstream sources (codex-rs, gemini-cli, etc.).

## Files

| File                   | Tool            | Spec Section |
| ---------------------- | --------------- | ------------ |
| `read_file.json`       | read_file       | 3.3          |
| `write_file.json`      | write_file      | 3.3          |
| `edit_file.json`       | edit_file       | 3.3          |
| `shell.json`           | shell           | 3.3          |
| `grep.json`            | grep            | 3.3          |
| `glob.json`            | glob            | 3.3          |
| `read_many_files.json` | read_many_files | 3.6 (Gemini)   |
| `list_dir.json`        | list_dir        | 3.6 (Gemini)   |
| `apply_patch.json`     | apply_patch     | App A (OpenAI) |
