"""
Module that defines the `Server` class
"""

import json
import sys

from ..Processor import Processor
from .jsonRpc import Request, Response
from .Logger import Logger

class Server(Logger):
    """
    Base class for all servers.
    """
    
    def __init__(self, processor: Processor = Processor()):
        self.processor = processor

    async def start(self) -> None:
        """
        Start this server.

        Starts listening for requests.
        """
        self.log(starting=True)
        await self.open()

    async def open(self) -> None:
        raise NotImplementedError()

    async def stop(self) -> None:
        """
        Stop this server.

        Stops listening for requests.
        """
        await self.close()
        self.log(stopped=True)

    async def close(self) -> None:
        raise NotImplementedError()

    async def receive(self, message: str):
        request = self.decode(message)
        response = Response(id=request.id, result="foo")
        self.log(request=request, response=response)
        return self.encode(response)

    def encode(self, response: Response) -> str:
        return json.dumps(response.__dict__)

    def decode(self, message: str) -> Request:
        return Request(**json.loads(message))
