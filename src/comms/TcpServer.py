import asyncio

from ..Processor import Processor
from .StreamMultiServer import StreamMultiServer

class TcpServer(StreamMultiServer):

    def __init__(self, processor: Processor, host: str = '127.0.0.1', port: int = 2000, encoders=None):
        StreamMultiServer.__init__(self, processor, encoders)
        self.host = host
        self.port = port

    @property
    def url(self):
        return f'tcp://{self.host}:{self.port}'

    async def open(self) -> None:
        await asyncio.start_server(self.on_client_connected, self.host, self.port)
