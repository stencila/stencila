from typing import Type
import gzip

from .CborEncoder import CborEncoder
from .jsonRpc import Request, Response, RequestOrResponse

class CborGzipEncoder(CborEncoder):

    def name(self) -> str:
        return 'cbor+gzip'

    def decode(self, message: bytes, cls: Type[RequestOrResponse]) -> RequestOrResponse:
        return CborEncoder.decode(self, gzip.decompress(message), cls)

    def encode(self, obj: RequestOrResponse) -> bytes:
        return gzip.compress(CborEncoder.encode(self, obj))
