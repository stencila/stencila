import asyncio
import os
import mmap
import string
import random

from .stencilaFiles import create_tempfile

# Protocol constants
MMAP_FLAG_EMPTY = b'0'
MMAP_FLAG_CLOSE = b'3'
MMAP_SIZE_BYTES = 4
MMAP_SIZE_ENDIAN = 'big'
MMAP_SIZE_SIGNED = False

class MmapConnection:

    def __init__(self, path=None, dir=None, size=1e8):
        if not path:
            path = create_tempfile(''.join(random.choices(string.ascii_lowercase + string.digits, k=32)))
            flag_read = b'1'
            flag_write = b'2'
        else:
            flag_read = b'2'
            flag_write = b'1'
        
        if not os.path.exists(path):
            file = open(path, 'wb')
            file.truncate(size)
        self._path = path
        self._size = int(size)
        
        self._flag_read = flag_read
        self._flag_write = flag_write

        file = open(self._path, 'r+b')
        self._map = mmap.mmap(file.fileno(), self._size)

    @property
    def path(self):
        return self._path

    @property
    def size(self):
        return self._size

    async def read(self, wait=1e-5):
        assert self._map
        while True:
            self._map.seek(0)
            flag = self._map.read(1)
            if flag == self._flag_read:
                size = int.from_bytes(self._map.read(MMAP_SIZE_BYTES), byteorder=MMAP_SIZE_ENDIAN, signed=MMAP_SIZE_SIGNED)
                if size:
                    message = self._map.read(size)
                    self._map.seek(0)
                    self._map.write(MMAP_FLAG_EMPTY)
                    return message
            elif flag == MMAP_FLAG_CLOSE:
                self._map.close()
                break

            if wait > 0:
                await asyncio.sleep(wait)
            else:
                break

    async def write(self, message: bytes) -> None:
        assert self._map
        self._map.seek(0)
        self._map.write(self._flag_write)
        self._map.write(len(message).to_bytes(length=MMAP_SIZE_BYTES, byteorder=MMAP_SIZE_ENDIAN, signed=MMAP_SIZE_SIGNED))
        self._map.write(message)

    def listen(self, callback) -> asyncio.Future:
        async def run():
            while True:
                message = await self.read()
                if not message:
                    break
                else:
                    callback(message)
        return asyncio.ensure_future(run())

    async def close(self) -> None:
        if self._map:
            self._map.seek(0)
            self._map.write(MMAP_FLAG_CLOSE)
            self._map.close()
            self._map = None
