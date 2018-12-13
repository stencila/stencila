from typing import Type
import base64

from .JsonGzipEncoder import JsonGzipEncoder
from .jsonRpc import RequestOrResponse

class JsonGzipBase64Encoder(JsonGzipEncoder):
    """
    Uses Base64 to encode Gzipped JSON so that messages can be transferred as text.

    According to Wikipedia, "Very roughly, the final size of Base64-encoded binary data is equal to 1.37 times the 
    original data size + 814 bytes (for headers)".
    """


    def name(self) -> str:
        return 'json+gzip+base64'

    def decode(self, message: bytes, cls: Type[RequestOrResponse]) -> RequestOrResponse:
        return JsonGzipEncoder.decode(self, base64.b64decode(message), cls)

    def encode(self, obj: RequestOrResponse) -> bytes:
        return base64.b64encode(JsonGzipEncoder.encode(self, obj))
