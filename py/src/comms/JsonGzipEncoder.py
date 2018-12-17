from typing import Type
import gzip

from .JsonEncoder import JsonEncoder
from .jsonRpc import Request, Response, RequestOrResponse

class JsonGzipEncoder(JsonEncoder):

    def name(self) -> str:
        return 'json+gzip'

    def decode(self, message: bytes, cls: Type[RequestOrResponse]) -> RequestOrResponse:
        return JsonEncoder.decode(self, gzip.decompress(message), cls)

    def encode(self, obj: RequestOrResponse) -> bytes:
        return gzip.compress(JsonEncoder.encode(self, obj))
