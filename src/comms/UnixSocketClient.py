from typing import Optional
import asyncio

from .StreamConnection import StreamConnection
from .StreamClient import StreamClient

class UnixSocketClient(StreamClient):

    def __init__(self, path: str):
        StreamClient.__init__(self)
        self.path = path

    @property
    def url(self):
        return f'unix://{self.path}'

    async def open(self) -> None:
        reader, writer = await asyncio.open_unix_connection(self.path)
        self.connection = StreamConnection(reader, writer)
        await StreamClient.open(self)
