# Example: Rust — extract function to reduce duplication

Target files: `src/parser/expression.rs`
Test command: `cargo test -p parser`
Refactoring focus: reduce duplication

**Discovery**: Read `src/parser/expression.rs` — three methods (`parse_binary`, `parse_unary`, `parse_call`) each contain a nearly identical 15-line block that constructs a `Span` from start and end positions, validates the span is non-empty, and wraps the result in a `Spanned<T>` struct. The surrounding codebase uses small helper functions for repeated patterns. The `Span` and `Spanned` types are defined in `src/parser/span.rs`.

**Analysis**:
- Duplication: 3 copies of the span-construction logic (high impact)
- The logic is identical except for the inner value type, which is already generic via `Spanned<T>`
- Extracting a helper is safe — the function is internal to the module, no public API change

**Action**:
1. Extract a `fn wrap_spanned<T>(start: Position, end: Position, value: T) -> Result<Spanned<T>, ParseError>` helper function
2. Place it in the `impl Parser` block near the existing private helpers, following the codebase's convention of grouping utility methods at the end of the impl block
3. Replace the three duplicated blocks with calls to `self.wrap_spanned(start, end, value)?`
4. Run `cargo check -p parser` — clean
5. Run `cargo test -p parser` — all 47 tests pass

**Summary**: Modified `src/parser/expression.rs`. Extracted `wrap_spanned` helper method, replacing 3 copies of 15-line span-construction logic with single-line calls. Category: duplication. No public API changes. All tests pass.
