import asyncio

class AsyncioConnection:


    def __init__(self, reader: asyncio.StreamReader, writer: asyncio.StreamWriter):
        self.reader = reader
        self.writer = writer

    def listen(self, callback) -> None:
        """
        Listen for messages on the connection.
        """
        async def listen():
            while True:
                line = await self.reader.readline()
                if line:
                    message = line.decode('utf8')
                    await callback(message)
                else:
                    break
        asyncio.ensure_future(listen())

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
        self.reader.feed_eof()
        self.writer.close()
