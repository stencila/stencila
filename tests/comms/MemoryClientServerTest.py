import pytest

from stencilaschema.comms.Client import Client
from stencilaschema.comms.Server import Server
from stencilaschema.comms.JsonEncoder import JsonEncoder
from stencilaschema.comms.JsonGzipEncoder import JsonGzipEncoder

from helpers.TestProcessor import TestProcessor

# Tests of in-memory client-server communication.
# Useful for testing methods in Client and Server base classes.

class MemoryClient(Client):

    def __init__(self, server, encoders = None):
        Client.__init__(self, encoders)
        self.server = server

    async def open(self) -> None:
        # Required to be implemented
        pass

    async def write(self, message: str) -> None:
        # Simulate writing a request to server and reading the reponse
        await self.read(await self.server.receive(message, self.encoder.name()))

    async def close(self) -> None:
        # Required to be implemented
        pass

class MemoryServer(Server):

    def __init__(self, processor):
        Server.__init__(self, processor, 
                        # Testing of alternative encoders
                        encoders=[JsonEncoder(), JsonGzipEncoder()])

    async def open(self) -> None:
        # Required to be implemented
        pass

    async def close(self) -> None:
        # Required to be implemened
        pass

thing1 = {'type': 'Thing', 'name': 'thing1'}
thing2 = {'type': 'Thing', 'name': 'thing2'}
thing3 = {'type': 'Thing', 'name': 'thing3'}

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
    assert client1.encoder.name() == 'json'
    await client1.start()
    assert client1.encoder.name() == 'json'
    assert await client1.execute(thing1) == thing1
    
    client2 = MemoryClient(server, encoders=[
        JsonEncoder(),
        JsonGzipEncoder()
    ])
    assert client2.encoder.name() == 'json'
    await client2.start()
    assert client2.encoder.name() == 'json'
    assert await client2.execute(thing1) == thing1
    
    client3 = MemoryClient(server, encoders=[
        JsonGzipEncoder(),
        JsonEncoder()
    ])
    assert client3.encoder.name() == 'json'
    await client3.start()
    assert client3.encoder.name() == 'json+gzip'
    assert await client3.execute(thing1) == thing1

    await client1.stop()
    await client2.stop()
    await client3.stop()
    await server.stop()
