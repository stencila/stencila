import asyncio
import os
import tempfile

import pytest

from stencilaschema.comms.UnixSocketClient import UnixSocketClient
from stencilaschema.comms.UnixSocketServer import UnixSocketServer

from helpers.TestProcessor import TestProcessor

@pytest.mark.asyncio
async def test_unix_socket():
    # Create test processor and a temporary file for the socket
    processor = TestProcessor()
    path = os.path.join(tempfile.mkdtemp(), 'socket')

    # Start the server and several clients listening to that file
    server = UnixSocketServer(processor, path)
    await server.start()
    client1 = UnixSocketClient(path)
    await client1.start()
    client2 = UnixSocketClient(path)
    await client2.start()
    client3 = UnixSocketClient(path)
    await client3.start()

    assert server.url == f'unix://{path}'
    assert client1.url == server.url
    assert client2.url == server.url
    assert client3.url == server.url

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
