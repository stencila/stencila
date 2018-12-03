"""
Module that defines the `Server` class
"""

import json

from ..Processor import Processor
from .jsonRpc import Request, Response

class Server:
    """
    Base class for all servers.
    """

    def __init__(self, processor: Processor = Processor(), logging=0):
        self.processor = processor
        self.logging = logging

    async def start(self) -> None:
        """
        Start this server.

        Starts listening for requests.
        """
        await self.open()

    async def stop(self) -> None:
        """
        Stop this server.

        Stops listening for requests.
        """
        await self.close()

    async def recieve(self, request: Request):
        response = Response(id=request.id, result="foo")
        await self.write(self.encode(response))

    async def open(self) -> None:
        raise NotImplementedError()

    async def close(self) -> None:
        raise NotImplementedError()

    def encode(self, response: Response) -> str:
        return json.dumps(response.__dict__)

    def decode(self, message: str) -> Request:
        return Request(**json.loads(message))

    async def read(self, message: str) -> None:
        await self.recieve(self.decode(message))

    async def write(self, message: str) -> None:
        raise NotImplementedError()
