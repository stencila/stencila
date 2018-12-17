from typing import List
import os
import re

from .stencilaFiles import create_tempdir
from .Client import Client
from .MmapMixin import MmapMixin

MMAP_URL_REGEX = re.compile(r'^mmap://(.+)')

class MmapClient(MmapMixin, Client):

    BYTE_WRITE = b'1'
    BYTE_READ = b'2'

    def __init__(self, url: str, encoders=None):
        MmapMixin.__init__(self)
        Client.__init__(self, url=url, encoders=None)

        match = MMAP_URL_REGEX.match(url)
        if match:
            self._path = match.group(1)
        else:
            raise RuntimeError(f'Invalid URL for memory mapped file: {url}')

    @staticmethod
    def connectable(url: str) -> bool:
        return url[:7] == 'mmap://'

    @staticmethod
    async def discover() -> List[Client]:
        clients: List[Client] = []
        tempdir = create_tempdir()
        for filename in os.listdir(tempdir):
            if filename.startswith('mmap-'):
                client = MmapClient('mmap://' + os.path.join(tempdir, filename))
                try:
                    await client.start()
                except Exception as exc:
                    raise exc # TODO: log these as warnings
                else:
                    clients.append(client)
        return clients
