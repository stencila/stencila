import json
import pytest

from stencilaschema.comms.jsonRpc import Request
from stencilaschema.comms.JsonEncoder import JsonEncoder
from stencilaschema.comms.JsonGzipEncoder import JsonGzipEncoder
from stencilaschema.comms.CborEncoder import CborEncoder
from stencilaschema.comms.CborGzipEncoder import CborGzipEncoder

from fixtures import fixture

encoders = dict(
    json=JsonEncoder(),
    jsongz=JsonGzipEncoder(),
    cbor=CborEncoder(),
    cborgz=CborGzipEncoder()
)

def read_fixture(filename):
    with open(fixture('requests', filename), 'r') as file:
        return JsonEncoder().decode(file.read(), Request)

requests = dict(
    record=read_fixture('record.json'),
    tree=read_fixture('tree.json'),
    table=read_fixture('table.json'),
    text=read_fixture('text.json')
)

matrix = {}
for ename, encoder in encoders.items():
    for rname, request in requests.items():
        matrix[f'{ename}-{rname}'] = (encoder, request)

sizes = {}

def roundtrip(encoder, request):
    message = encoder.encode(request)
    encoder.decode(message, Request)
    return len(message)

@pytest.mark.parametrize('name', list(matrix.keys()))
def test(benchmark, name):
    encoder, request = matrix[name]
    sizes[name] = benchmark(roundtrip, encoder, request)

def test_print_sizes():
    print(sizes)
