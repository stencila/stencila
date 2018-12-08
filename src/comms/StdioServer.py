import asyncio
import sys

from .AsyncioConnection import AsyncioConnection
from .Server import Server
from .StdioMixin import StdioMixin

class StdioServer(Server, StdioMixin):

    async def open(self) -> None:
        """
        Create an async connection on stdin / stdout
        """

        self.connection = await AsyncioConnection.from_files(sys.stdin, sys.stdout)
        async def callback(message):
            await self.connection.write(await self.receive(message))
        assert self.connection
        self.connection.listen(callback)

    async def close(self) -> None:
        """
        Close any connection
        """

        if self.connection:
            await self.connection.close()
