from typing import Type

import json

JSON_ENCODING = {
    'contentType': 'application/json'
}

class JsonEncoder:

    @staticmethod
    def decode(message: bytes, cls: Type):
        request = cls()
        dic = json.loads(message)
        request.__dict__.update(dic)
        return request

    @staticmethod
    def encode(obj) -> bytes:
        return json.dumps(obj, cls=JSONEncoderExtension).encode('utf8')


class JSONEncoderExtension(json.JSONEncoder):
    """
    Extension of the builtin JSON encoder to handle objects
    """

    def default(self, obj):
        return obj.__dict__

