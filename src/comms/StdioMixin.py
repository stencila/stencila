import asyncio

class StdioMixin:

    def __init__(self, input, output):
        self.input = input
        self.output = output
        self.reader = None
        self.writer = None

    async def open(self) -> None:
        loop = asyncio.get_event_loop()
        
        if self.reader is None:
            self.reader = asyncio.StreamReader()
            reader_protocol = asyncio.StreamReaderProtocol(self.reader)
            await loop.connect_read_pipe(lambda: reader_protocol, self.input)

        if self.writer is None:
            writer_transport, writer_protocol = await loop.connect_write_pipe(asyncio.streams.FlowControlMixin, self.output)
            self.writer = asyncio.streams.StreamWriter(writer_transport, writer_protocol, self.reader, loop)

        async def do():
            while True:
                line = await self.reader.readline()
                if line:
                    message = line.decode('utf8')
                    await self.read(message)
                else:
                    break
        await do()

    async def write(self, message: str) -> None:
        line = message + '\n'
        bites = line.encode('utf8')
        self.writer.write(bites)
        await self.writer.drain()
