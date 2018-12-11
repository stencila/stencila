from typing import Type
import gzip

from .JsonEncoder import JsonEncoder

JSON_GZIP_ENCODING = {
    'contentType': 'application/json',
    'contentEncoding': 'gzip'
}

class JsonGzipEncoder(JsonEncoder):

    @staticmethod
    def decode(message: bytes, cls: Type):
        return JsonEncoder.decode(gzip.decompress(message), cls)

    @staticmethod
    def encode(obj) -> bytes:
        return gzip.compress(JsonEncoder.encode(obj))
