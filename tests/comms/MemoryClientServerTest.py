import pytest

from stencilaschema.comms.Client import Client
from stencilaschema.comms.Server import Server

from helpers.TestProcessor import TestProcessor

# Tests of in-memory client-server communication.
# Useful for testing methods in Client and Server base classes.

class MemoryClient(Client):

    def __init__(self, server):
        self.server = server

    async def open(self) -> None:
        pass

    async def write(self, message: str) -> None:
        await self.read(await self.server.receive(message))

    async def close(self) -> None:
        pass

class MemoryServer(Server):

    async def open(self) -> None:
        pass

    async def close(self) -> None:
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
