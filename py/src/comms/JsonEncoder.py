from typing import Type
import json

from .Encoder import Encoder
from .jsonRpc import Request, Response, RequestOrResponse

class JsonEncoder(Encoder):

    def name(self) -> str:
        return 'json'

    def decode(self, message: bytes, cls: Type[RequestOrResponse]) -> RequestOrResponse:
        instance = cls()
        dic = json.loads(message)
        instance.__dict__.update(dic)
        return instance

    def encode(self, obj: RequestOrResponse) -> bytes:
        return json.dumps(obj, cls=JSONEncoderExtension).encode('utf8')


class JSONEncoderExtension(json.JSONEncoder):
    """
    Extension of the builtin JSON encoder to handle objects
    """

    def default(self, obj):
        return obj.__dict__
