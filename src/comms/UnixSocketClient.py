from typing import Optional
import asyncio

from .AsyncioConnection import AsyncioConnection
from .Client import Client
from .UnixSocketMixin import UnixSocketMixin

class UnixSocketClient(UnixSocketMixin, Client):

    connection: Optional[AsyncioConnection]

    def __init__(self, path):
        UnixSocketMixin.__init__(self, path)
        Client.__init__(self)

        self.connection = None

    async def open(self) -> None:
        reader, writer = await asyncio.open_unix_connection(self.path)
        self.connection = AsyncioConnection(reader, writer)
        self.connection.listen(self.read)

    async def write(self, message: bytes) -> None:
        assert self.connection
        await self.connection.write(message)

    async def close(self) -> None:
        if self.connection:
            await self.connection.close()
