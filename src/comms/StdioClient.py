import asyncio
import sys

from .Client import Client
from .StdioMixin import StdioMixin

class StdioClient(StdioMixin, Client):

    def __init__(self, input=sys.stdin, output=sys.stdout):
        StdioMixin.__init__(self, input, output)
        Client.__init__(self)

        self.subprocess = None

    async def spawn(self, cmd):
        self.subprocess = await asyncio.create_subprocess_exec(
            *cmd,
            stdin=asyncio.subprocess.PIPE, 
            stdout=asyncio.subprocess.PIPE
        )
        self.reader = self.subprocess.stdout
        self.writer = self.subprocess.stdin

        await self.start()

    async def kill(self):
        self.subprocess.kill()
