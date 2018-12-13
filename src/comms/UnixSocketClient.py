import asyncio
import re

from .StreamConnection import StreamConnection
from .StreamClient import StreamClient

UNIX_URL_REGEX = re.compile(r'^unix://(.+)')

class UnixSocketClient(StreamClient):

    def __init__(self, url: str, encoders=None):
        StreamClient.__init__(self, encoders=None)
        match = UNIX_URL_REGEX.match(url)
        if match:
            self._url = url
            self._path = match.group(1)
        else:
            raise RuntimeError(f'Invalid URL for UNIX: {url}')

    @property
    def url(self):
        return self._url

    async def open(self) -> None:
        reader, writer = await asyncio.open_unix_connection(self._path)
        self.connection = StreamConnection(reader, writer)
        await StreamClient.open(self)
