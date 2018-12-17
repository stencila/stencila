from typing import Type
import cbor

from .Encoder import Encoder
from .jsonRpc import RequestOrResponse

class CborEncoder(Encoder):

    def name(self) -> str:
        return 'cbor'

    def decode(self, message: bytes, cls: Type[RequestOrResponse]) -> RequestOrResponse:
        obj = cls()
        dic = cbor.loads(message)
        obj.__dict__.update(dic)
        return obj

    def encode(self, obj: RequestOrResponse) -> bytes:
        return cbor.dumps(obj.__dict__)
