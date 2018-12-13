import sys

from .StreamConnection import StreamConnection
from .StreamServer import StreamServer

class StdioServer(StreamServer):

    async def open(self) -> None:
        """
        Create an async connection on stdin / stdout.
        """
        self.connection = await StreamConnection.from_files(sys.stdin, sys.stdout)
        await StreamServer.open(self)
