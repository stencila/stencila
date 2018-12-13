import asyncio

from ..Processor import Processor
from .StreamMultiServer import StreamMultiServer

class UnixSocketServer(StreamMultiServer):
    """
    A Server communicating over UNIX domain sockets
    """

    def __init__(self, processor: Processor, path: str):
        StreamMultiServer.__init__(self, processor)
        self.path = path

    @property
    def url(self):
        return f'unix://{self.path}'

    async def open(self) -> None:
        """
        Start the UNIX socket server and create an
        async connections when a client connects.
        """
        await asyncio.start_unix_server(self.on_client_connected, self.path)
