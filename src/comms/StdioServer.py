import asyncio
import sys

from .AsyncioConnection import AsyncioConnection
from .Server import Server
from .StdioMixin import StdioMixin

class StdioServer(Server, StdioMixin):

    async def open(self) -> None:
        # Create async reader and writer on stdin and stdout
        # See https://gist.github.com/nathan-hoad/8966377
        
        loop = asyncio.get_event_loop()
        
        reader = asyncio.StreamReader()
        reader_protocol = asyncio.StreamReaderProtocol(reader)
        await loop.connect_read_pipe(lambda: reader_protocol, sys.stdout)

        writer_transport, writer_protocol = await loop.connect_write_pipe(asyncio.streams.FlowControlMixin, sys.stdout)
        writer = asyncio.streams.StreamWriter(writer_transport, writer_protocol, reader, loop)

        # Create a connection using reader and writer and listen on it
        self.connection = AsyncioConnection(reader, writer)
        async def callback(message):
            await self.connection.write(await self.receive(message))
        self.connection.listen(callback)

    async def close(self) -> None:
        if self.connection:
            await self.connection.close()
