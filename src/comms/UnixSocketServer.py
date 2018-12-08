from typing import List
import asyncio

from ..Processor import Processor
from .AsyncioConnection import AsyncioConnection
from .Server import Server
from .UnixSocketMixin import UnixSocketMixin

class UnixSocketServer(UnixSocketMixin, Server):
    """
    A Server communicating over UNIX domain sockets
    """

    """
    List of client connections
    """
    connections: List[AsyncioConnection]

    def __init__(self, processor: Processor, path: str):
        UnixSocketMixin.__init__(self, path)
        Server.__init__(self, processor)
        self.connections = []

    async def open(self) -> None:
        """
        Start the UNIX socket server and create an
        async connections when a client connects.
        """
        def on_client_connected(reader, writer):
            self.log(connection=True)
            connection = AsyncioConnection(reader, writer)
            async def callback(message):
                await connection.write(await self.receive(message))
            connection.listen(callback)
            self.connections.append(connection)
        await asyncio.start_unix_server(on_client_connected, self.path)

    async def close(self) -> None:
        """
        Close all the client connections
        """
        for connection in self.connections:
            await connection.close()
