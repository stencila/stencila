# Example: Python — naming improvements and style alignment

Target files: `mylib/transform.py`
Test command: `pytest tests/test_transform.py -v`
Refactoring focus: improve naming, align with codebase style

**Discovery**: Read sibling modules (`mylib/config.py`, `mylib/validate.py`). The codebase uses descriptive function names with verb prefixes (`parse_config`, `validate_schema`, `load_defaults`), type hints on all function signatures, and early returns with guard clauses instead of nested conditionals. Read `mylib/transform.py` — the module works correctly but was written hastily.

**Analysis**:
- Naming: `def do_it(d, k, v)` should be `def apply_transform(data: dict, key: str, value: Any) -> dict` — parameters are single letters, function name is vague
- Naming: `temp`, `res`, `flag` variables should be `transformed_entry`, `result`, `should_replace`
- Complexity: a 4-level nested if/else block can be flattened using guard clauses (consistent with `validate.py` style)
- Style: missing type hints on two functions, inconsistent with the rest of the codebase
- Note: `do_it` is only called internally via `_process_batch`, not part of the public API — renaming is safe

**Action**:
1. Rename `do_it` to `apply_transform` and update its parameters to have descriptive names with type hints
2. Update the single call site in `_process_batch` to use `apply_transform`
3. Rename local variables `temp` → `transformed_entry`, `res` → `result`, `flag` → `should_replace`
4. Flatten the nested conditional using two guard clauses with early returns
5. Add type hints to `_process_batch` and `_merge_results` (the two functions missing them)
6. Run `python -m py_compile mylib/transform.py` — clean
7. Run `pytest tests/test_transform.py -v` — all 12 tests pass

**Summary**: Modified `mylib/transform.py`. Renamed `do_it` → `apply_transform` with descriptive parameters (naming). Renamed 3 local variables for clarity (naming). Flattened 4-level nested conditional to guard clauses (complexity). Added type hints to 2 functions (style). No public API changes — all renamed items are internal. All tests pass.
