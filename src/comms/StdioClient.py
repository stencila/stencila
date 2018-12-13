from typing import Optional
import asyncio

from .StreamConnection import StreamConnection
from .StreamClient import StreamClient

class StdioClient(StreamClient):

    subprocess: Optional[asyncio.subprocess.Process]

    def __init__(self, command: str):
        StreamClient.__init__(self)

        self.command = command

    async def open(self) -> None:
        # Start subprocess and wait on it
        self.subprocess = await asyncio.create_subprocess_exec(
            *self.command,
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE
        )
        asyncio.ensure_future(self.subprocess.wait())
        
        # Create an async connection to the subprocess
        reader = self.subprocess.stdout
        writer = self.subprocess.stdin
        assert reader and writer
        self.connection = StreamConnection(reader, writer)
        await StreamClient.open(self)

    async def close(self) -> None:
        await StreamClient.close(self)
        if self.subprocess:
            self.subprocess.kill()
