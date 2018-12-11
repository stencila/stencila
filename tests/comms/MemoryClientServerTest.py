import pytest

from stencilaschema.comms.Client import Client
from stencilaschema.comms.Server import Server
from stencilaschema.comms.JsonEncoder import JSON_ENCODING
from stencilaschema.comms.JsonGzipEncoder import JSON_GZIP_ENCODING

from helpers.TestProcessor import TestProcessor

# Tests of in-memory client-server communication.
# Useful for testing methods in Client and Server base classes.

class MemoryClient(Client):

    def __init__(self, server, encodings = None):
        self.server = server
        self.encodings = encodings

    async def open(self) -> None:
        # Required to be implemented
        pass

    async def hello(self) -> None:
        # Override the base method to provide encodings
        await Client.hello(self, encodings=self.encodings)

    async def write(self, message: str) -> None:
        # Simulate writing a request to server and reading the reponse
        await self.read(await self.server.receive(message, self.encoding))

    async def close(self) -> None:
        # Required to be implemented
        pass

class MemoryServer(Server):

    def __init__(self, processor):
        Server.__init__(self, processor)
        self.connections = []

    async def open(self) -> None:
        # Required to be implemented
        pass

    async def close(self) -> None:
        # Required to be implemened
        pass


@pytest.mark.asyncio
async def test_memory():
    # Start the server and several clients listening it
    processor = TestProcessor()
    server = MemoryServer(processor)
    await server.start()

    client1 = MemoryClient(server)
    await client1.start()
    client2 = MemoryClient(server)
    await client2.start()
    client3 = MemoryClient(server)
    await client3.start()
    
    thing1 = {'type': 'Thing', 'name': 'thing1'}
    thing2 = {'type': 'Thing', 'name': 'thing2'}
    thing3 = {'type': 'Thing', 'name': 'thing3'}

    assert await client1.execute(thing1) == thing1
    assert await client2.execute(thing2) == thing2
    assert await client3.execute(thing3) == thing3

    # Stop everything
    await client1.stop()
    await client2.stop()
    await client3.stop()
    await server.stop()

@pytest.mark.asyncio
async def test_encodings():
    processor = TestProcessor()
    server = MemoryServer(processor)
    await server.start()

    client1 = MemoryClient(server)
    await client1.start()
    assert client1.encoding == JSON_ENCODING
    
    
    client2 = MemoryClient(server, [
        JSON_ENCODING,
        JSON_GZIP_ENCODING
    ])
    await client2.start()
    assert client2.encoding == JSON_ENCODING
    
    client3 = MemoryClient(server, [
        {}, # Dummy encoding spec
        JSON_GZIP_ENCODING,
        JSON_ENCODING
    ])
    await client3.start()
    assert client3.encoding == JSON_GZIP_ENCODING

    await client1.stop()
    await client2.stop()
    await client3.stop()
    await server.stop()
