# Example: Python package with pytest

Slice: "Phase 1 / Slice 2" — Parse configuration from TOML
Acceptance criteria: `parse_config` returns a `Config` object with `name`, `version`; raises `ConfigError` for invalid TOML
Package: `python/stencila`

**Discovery**: Exploration finds `tests/test_*.py` files using pytest assertions. `conftest.py` exists with shared fixtures. `pyproject.toml` has a `[tool.pytest.ini_options]` section. The test command is `pytest python/stencila/tests/`.

Test written in `python/stencila/tests/test_config.py`:

```python
import pytest
from stencila.config import Config, ConfigError, parse_config


def test_parse_config_returns_config_with_name_and_version():
    raw = '[project]\nname = "example"\nversion = "1.0.0"'
    config = parse_config(raw)
    assert isinstance(config, Config)
    assert config.name == "example"
    assert config.version == "1.0.0"


def test_parse_config_raises_config_error_for_invalid_toml():
    with pytest.raises(ConfigError):
        parse_config("this is not valid toml [[[")
```

Context stored:

- `slice.test_files` = `python/stencila/tests/test_config.py`
- `slice.test_command` = `pytest python/stencila/tests/test_config.py`
