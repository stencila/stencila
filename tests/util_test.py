from python.util import toJSON

def test_toJSON():
    assert toJSON(True) == "true"
