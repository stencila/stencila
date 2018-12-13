from typing import Optional
import asyncio
import re

from .StreamConnection import StreamConnection
from .StreamClient import StreamClient

TCP_URL_REGEX = re.compile(r'^tcp://([^:/]+)(?:\:(\d+))?')

class TcpClient(StreamClient):

    def __init__(self, url: str = 'tcp://127.0.0.1', encoders=None):
        StreamClient.__init__(self, encoders=encoders)
        match = TCP_URL_REGEX.match(url)
        if match:
            self._url = url
            self._host = match.group(1)
            self._port = int(match.group(2)) if match.group(2) else 2000
        else:
            raise RuntimeError(f'Invalid URL for TCP: {url}')

    @property
    def url(self):
        return self._url

    async def open(self) -> None:
        reader, writer = await asyncio.open_connection(self._host, self._port)
        self.connection = StreamConnection(reader, writer)
        await StreamClient.open(self)
