from stencila import stencila_types as T
from stencila.utilities import from_json, to_json


def test_construction():
    a = T.Executable()


def test_loads():
    a_json = """{
        "type": "Article",
        "content": [{
            "type": "Paragraph", "content": [
                {"type": "Text", "value": "Hello world"
            }
        ]}]}
    """
    a = from_json(a_json)
    a_json_out = to_json(a)
    a2 = from_json(a_json_out)
    assert a == a2

    # print(txt)
    # node = from_json(txt)
    # print(node)
