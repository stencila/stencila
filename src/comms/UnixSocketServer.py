from typing import List
import asyncio
import sys

from .AsyncioConnection import AsyncioConnection
from .Server import Server
from .UnixSocketMixin import UnixSocketMixin

class UnixSocketServer(Server, UnixSocketMixin):

    connections: List[AsyncioConnection]

    def __init__(self, path):
        Server.__init__(self)
        UnixSocketMixin.__init__(self, path)

        self.connections = []

    async def open(self) -> None:
        # Callback for each message that is read
        def on_client_connected(reader, writer):
            self.log(connection=True)
            connection = AsyncioConnection(reader, writer)
            async def callback(message):
                await connection.write(await self.receive(message))
            connection.listen(callback)
            self.connections.append(connection)
        await asyncio.start_unix_server(on_client_connected, self.path)

    async def close(self) -> None:
        for connection in self.connections:
            await connection.close()
