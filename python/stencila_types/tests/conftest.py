from dataclasses import dataclass
from pathlib import Path

import pytest

ROOT_PATH = Path(__file__).parent.parent.parent.parent
NODES_PATH = ROOT_PATH / "examples" / "nodes"


@dataclass(repr=False)
class JsonExample:
    path: Path

    @property
    def name(self):
        return self.path.stem

    def __repr__(self):
        return self.name


JSON_PATHS = [JsonExample(p) for p in NODES_PATH.rglob("*.json")]
JSON_IDS = [j.name for j in JSON_PATHS]


@pytest.fixture(scope="session", params=JSON_PATHS, ids=JSON_IDS)
def json_example(request):
    return request.param
