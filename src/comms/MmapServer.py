import asyncio
import mmap
import random
import string

from .stencilaFiles import create_tempfile
from .MmapMixin import MmapMixin
from .Server import Server

class MmapServer(MmapMixin, Server):

    BYTE_WRITE = b'2'
    BYTE_READ = b'1'

    def __init__(self, processor=None, encoders=None):
        MmapMixin.__init__(self)
        Server.__init__(self, processor=processor, encoders=None)
        self._id = None

    @property
    def url(self) -> str:
        return f'mmap://{self._path}'

    async def open(self) -> None:
        self._id = 'mmap-py-' + ''.join(random.choices(string.ascii_lowercase + string.digits, k=32))
        self._path = create_tempfile(self._id)
        file = open(self._path, 'w+b')
        file.truncate(1e6)
        file.close()

        await MmapMixin.open(self)

    async def read(self, message: bytes) -> None:
        await self.write(await self.receive(message, 'json'))
