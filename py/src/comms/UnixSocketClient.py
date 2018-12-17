from typing import List
import asyncio
import os
import re

from .stencilaFiles import create_tempdir
from .StreamConnection import StreamConnection
from .StreamClient import StreamClient

UNIX_URL_REGEX = re.compile(r'^unix://(.+)')

class UnixSocketClient(StreamClient):

    def __init__(self, url: str, encoders=None):
        StreamClient.__init__(self, url=url, encoders=None)
        match = UNIX_URL_REGEX.match(url)
        if match:
            self._path = match.group(1)
        else:
            raise RuntimeError(f'Invalid URL for UNIX domain socket: {url}')

    @staticmethod
    def connectable(url: str) -> bool:
        return url[:7] == 'unix://'

    @staticmethod
    async def discover() -> List['Client']:
        clients = []
        tempdir = create_tempdir()
        for filename in os.listdir(tempdir):
            if filename.startswith('unix-'):
                client = UnixSocketClient('unix://' + os.path.join(tempdir, filename))
                try:
                    await client.start()
                except ConnectionRefusedError as exc:
                    pass # TODO log(message=str(exc))
                else:
                    clients.append(client)
        return clients

    async def open(self) -> None:
        reader, writer = await asyncio.open_unix_connection(self._path)
        self.connection = StreamConnection(reader, writer)
        await StreamClient.open(self)
