from typing import Optional
import asyncio

from .AsyncioConnection import AsyncioConnection
from .Client import Client
from .StdioMixin import StdioMixin

class StdioClient(StdioMixin, Client):

    subprocess: Optional[asyncio.subprocess.Process]

    def __init__(self, command: str):
        StdioMixin.__init__(self)
        Client.__init__(self)

        self.command = command

    async def open(self) -> None:
        # Start subprocess
        self.subprocess = await asyncio.create_subprocess_exec(
            *self.command,
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE
        )
        # Create an async connection to the subprocess
        reader = self.subprocess.stdout
        writer = self.subprocess.stdin
        assert reader and writer
        self.connection = AsyncioConnection(reader, writer)
        self.connection.listen(self.read)
        # Wait on the subprocess
        asyncio.ensure_future(self.subprocess.wait())

    async def write(self, message: bytes) -> None:
        assert self.connection
        await self.connection.write(message)

    async def close(self) -> None:
        if self.connection:
            await self.connection.close()
        if self.subprocess:
            self.subprocess.kill()
