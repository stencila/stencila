import pytest

from stencila_types.utilities import from_json, to_json


@pytest.mark.skip_relaxed_json(
    lambda json_example: json_example.name() == "list",
)
def test_load_json_example(json_example):
    # Load
    node1 = from_json(json_example.path.read_text())

    # Round trip
    json_str = to_json(node1)
    node2 = from_json(json_str)

    # Check we're good
    assert node1 == node2
