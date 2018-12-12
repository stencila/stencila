
from stencilaschema.comms.jsonRpc import Request, Response
from stencilaschema.comms.JsonGzipBase64Encoder import JsonGzipBase64Encoder

def test():
    encoder = JsonGzipBase64Encoder()

    request1 = Request(id=1, method='foo', params=['bar'])
    request2 = encoder.decode(encoder.encode(request1), Request)
    assert request2.__dict__ == request1.__dict__

    response1 = Response(id=1, result='baz', error=None)
    response2 = encoder.decode(encoder.encode(response1), Response)
    assert response2.__dict__ == response1.__dict__
