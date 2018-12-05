from typing import Optional
import asyncio

from .AsyncioConnection import AsyncioConnection
from .Client import Client
from .UnixSocketMixin import UnixSocketMixin

class UnixSocketClient(Client, UnixSocketMixin):

    connection: Optional[AsyncioConnection]

    def __init__(self, path):
        Client.__init__(self)
        UnixSocketMixin.__init__(self, path)

        self.connection = None

    async def open(self) -> None:
        reader, writer = await asyncio.open_unix_connection(self.path)
        self.connection = AsyncioConnection(reader, writer)
        self.connection.listen(self.read)

    async def write(self, message: str) -> None:
        assert self.connection
        await self.connection.write(message)

    async def close(self) -> None:
        if self.connection:
            await self.connection.close()
