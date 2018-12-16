from typing import Type

from .jsonRpc import RequestOrResponse

class Encoder:

    def name(self) -> str:
        raise NotImplementedError()

    def decode(self, message: bytes, cls: Type[RequestOrResponse]) -> RequestOrResponse:
        raise NotImplementedError()

    def encode(self, obj: RequestOrResponse) -> bytes:
        raise NotImplementedError()
