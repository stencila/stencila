from dataclasses import dataclass
from pathlib import Path

import pytest

ROOT_PATH = Path(__file__).parent.parent.parent.parent
NODES_PATH = ROOT_PATH / "examples" / "nodes"


@dataclass(repr=False)
class JsonExample:
    path: Path

    def name(self):
        return self.path.stem


JSON_PATHS = [JsonExample(p) for p in NODES_PATH.rglob("*.json")]
# JSON_IDS = [j.name for j in JSON_PATHS]


@pytest.fixture(scope="session", params=JSON_PATHS, ids=JsonExample.name)
def json_example(request):
    return request.param


# https://stackoverflow.com/questions/73463001/
# how-to-skip-parametrized-tests-with-pytest
def pytest_runtest_setup(item):
    """This allows us to see what we have skipped in the tests"""
    skip_funcs = [mark.args[0] for mark in item.iter_markers(name="skip_relaxed_json")]
    if any(f(**item.callspec.params) for f in skip_funcs):
        pytest.skip("Skipping relaxed JSON example")
