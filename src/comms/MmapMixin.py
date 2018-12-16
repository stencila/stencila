import asyncio
import mmap

# Protocol for message size bytes
SIZE_BYTES = 4
SIZE_ENDIANNESS = 'big'
SIZE_SIGNED = False

# First byte in file is a flag byte
BYTE_NONE = b'0' # Value when no message to be read

# Time to sleep between polling the flag byte
LISTEN_SLEEP = 1e-5

class MmapMixin:

    def __init__(self):
        self._path = None
        self._mmap = None
        self._task = None

    async def open(self) -> None:
        file = open(self._path, 'r+b')
        self._mmap = mmap.mmap(file.fileno(), 0)
        async def listen():
            while True:
                self._mmap.seek(0)
                flag = self._mmap.read(1)
                if flag == self.BYTE_READ:
                    size = int.from_bytes(self._mmap.read(SIZE_BYTES), byteorder=SIZE_ENDIANNESS, signed=SIZE_SIGNED)
                    if size:
                        message = self._mmap.read(size)
                        self._mmap.seek(0)
                        self._mmap.write(BYTE_NONE)
                        await self.read(message)
                else:
                    await asyncio.sleep(LISTEN_SLEEP)
        self._task = asyncio.ensure_future(listen())

    async def write(self, message: bytes) -> None:
        assert self._mmap
        self._mmap.seek(0)
        self._mmap.write(self.BYTE_WRITE)
        self._mmap.write(len(message).to_bytes(length=4, byteorder=SIZE_ENDIANNESS, signed=SIZE_SIGNED))
        self._mmap.write(message)

    async def close(self) -> None:
        if self._task:
            self._task.cancel()
            try:
                await self._task
            except asyncio.CancelledError:
                assert self._task.cancelled()
                self._task = None
        if self._mmap:
            self._mmap.close()
