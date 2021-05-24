from stencila.schema.util import node_type

def test_node_type():
    assert node_type(None) == "Null"
    assert node_type(True) == "Boolean"
    assert node_type(42) == "Number"
    assert node_type(3.14) == "Number"
    assert node_type("Hello") == "Text"
    assert node_type([]) == "Array"
    assert node_type(tuple()) == "Array"
    assert node_type({}) == "Object"
    assert node_type({"type": "CodeChunk"}) == "CodeChunk"
