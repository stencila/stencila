# Example: Python — new module

Slice: "Phase 1 / Slice 2" — Parse configuration from TOML
Acceptance criteria: `parse_csv` function returns a list of row dicts; raises `ParseError` on empty file
Package: `mylib/`

**Test failure output**:
```
FAILED tests/test_parser.py::test_parse_csv_returns_rows - ModuleNotFoundError: No module named 'mylib.parser'
FAILED tests/test_parser.py::test_parse_csv_empty_file_raises - ModuleNotFoundError: No module named 'mylib.parser'
```

**Discovery**: Read `mylib/__init__.py` and sibling modules (`mylib/config.py`, `mylib/utils.py`). The package uses explicit re-exports in `__init__.py`. Functions use type hints, raise custom exception classes that inherit from a base `MyLibError`, and follow Google-style docstrings.

**Action**:
1. Read `tests/test_parser.py` — imports `from mylib.parser import parse_csv`
2. Tests show `parse_csv` takes a `str` argument and returns `list[dict[str, str]]`
3. Create `mylib/parser.py` with `parse_csv` function and `ParseError` class matching test expectations
4. Update `mylib/__init__.py` to add re-export of `parse_csv` (following existing pattern for `parse_config`)
5. Run `python -m py_compile mylib/parser.py` — clean

**Summary**: Created `mylib/parser.py` with `parse_csv` function and `ParseError` exception. Modified `mylib/__init__.py` to add re-export. Followed existing module conventions: type hints, custom exception inheriting from `MyLibError`, Google-style docstring.
