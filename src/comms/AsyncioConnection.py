from typing import Optional, Any
import asyncio

class AsyncioConnection:

    task: Optional[asyncio.Future[Any]]

    def __init__(self, reader: asyncio.StreamReader, writer: asyncio.StreamWriter):
        self.reader = reader
        self.writer = writer
        self.task = None

    @staticmethod
    async def from_files(input, output):
        # Create async reader and writer on stdin and stdout
        # See https://gist.github.com/nathan-hoad/8966377
        
        loop = asyncio.get_event_loop()
        
        reader = asyncio.StreamReader()
        reader_protocol = asyncio.StreamReaderProtocol(reader)
        await loop.connect_read_pipe(lambda: reader_protocol, input)

        writer_transport, writer_protocol = await loop.connect_write_pipe(asyncio.streams.FlowControlMixin, output)
        writer = asyncio.streams.StreamWriter(writer_transport, writer_protocol, reader, loop)

        return AsyncioConnection(reader, writer)

    def listen(self, callback) -> None:
        """
        Listen for messages on the connection.
        """
        async def listen():
            try:
                while True:
                    line = await self.reader.readline()
                    if line:
                        message = line.decode('utf8')
                        await callback(message)
                    else:
                        break
            except asyncio.CancelledError:
                raise
        self.task = asyncio.ensure_future(listen())

    async def finish(self) -> None:
        if self.task:
            self.task.cancel()
            try:
                await self.task
            except asyncio.CancelledError:
                assert self.task.cancelled()
                self.task = None

    async def write(self, message: str) -> None:
        """
        Write a message to the connection.
        """
        line = message + '\n'
        bites = line.encode('utf8')
        self.writer.write(bites)
        await self.writer.drain()

    async def close(self) -> None:
        """
        Close the connection.
        """
        await self.finish()
        self.reader.feed_eof()
        self.writer.close()
