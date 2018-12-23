import asyncio
import os
import string
import random

from .stencilaFiles import create_tempfile

# Protocol constants
STREAM_LENGTH_BYTES = 4
STREAM_LENGTH_ENDIAN = 'big'
STREAM_LENGTH_SIGNED = False

class StreamConnection:

    def __init__(self, reader: asyncio.StreamReader, writer: asyncio.StreamWriter):
        self.reader = reader
        self.writer = writer

    @staticmethod
    async def from_files(input, output):
        loop = asyncio.get_event_loop()

        reader = asyncio.StreamReader()
        reader_protocol = asyncio.StreamReaderProtocol(reader)
        await loop.connect_read_pipe(lambda: reader_protocol, input)

        writer_transport, writer_protocol = await loop.connect_write_pipe(asyncio.streams.FlowControlMixin, output)
        writer = asyncio.streams.StreamWriter(writer_transport, writer_protocol, reader, loop)

        return StreamConnection(reader, writer)

    async def read_n(self, n: int) -> bytes:
        """Read exactly n bytes from the reader.
        """
        buffer = b''
        while n > 0:
            data = await self.reader.read(n)
            if not data:
                raise RuntimeError('Connection closed unexpectedly')
            buffer += data
            n -= len(data)
        return buffer

    async def read(self, wait: float = 1e-5) -> bytes:
        """Read a message from the connection
        
        :param wait: Time to wait between read retries, defaults to 1e-5
        :param wait: float, optional
        :return: The message
        :rtype: bytes
        """
        while True:
            message_length_bytes = await self.read_n(STREAM_LENGTH_BYTES)
            message_length = int.from_bytes(message_length_bytes, byteorder=STREAM_LENGTH_ENDIAN, signed=STREAM_LENGTH_SIGNED)
            if message_length:
                message = await self.read_n(message_length)
                return message
            if wait > 0:
                await asyncio.sleep(wait)
            else:
                break

    async def write(self, message: bytes) -> None:
        message_length = len(message).to_bytes(length=STREAM_LENGTH_BYTES, byteorder=STREAM_LENGTH_ENDIAN, signed=STREAM_LENGTH_SIGNED)
        self.writer.write(message_length)
        self.writer.write(message)

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
