import asyncio

from ..Processor import Processor
from .StreamMultiServer import StreamMultiServer

class TcpServer(StreamMultiServer):

    def __init__(self, processor: Processor, host: str = '127.0.0.1', port: int = 2000, encoders=None):
        StreamMultiServer.__init__(self, processor, encoders)
        self._host = host
        self._port = port

    @property
    def url(self):
        return f'tcp://{self._host}:{self._port}'

    async def open(self) -> None:
        try:
            await asyncio.start_server(self.on_client_connected, self._host, self._port)
        except OSError as exc:
            if 'address already in use' in str(exc):
                # Port is already being used, try again with the next port
                self._port += 1
                await self.open()
            else:
                raise exc
