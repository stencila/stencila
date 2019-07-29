from stencila.schema.util import to_json

def test_to_json():
    assert to_json(True) == "true"
