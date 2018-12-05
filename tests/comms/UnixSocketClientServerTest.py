import asyncio
import os
import tempfile

import pytest

from stencilaschema.comms.UnixSocketClient import UnixSocketClient
from stencilaschema.comms.UnixSocketServer import UnixSocketServer


@pytest.mark.asyncio
async def test_unix_socket():
    # Create a temporary file for the socket
    path = os.path.join(tempfile.mkdtemp(), 'socket')

    # Start the server and several clients listening to that file
    server = UnixSocketServer(path)
    await server.start()
    client1 = UnixSocketClient(path)
    await client1.start()
    client2 = UnixSocketClient(path)
    await client2.start()
    client3 = UnixSocketClient(path)
    await client3.start()

    # Stop everything
    await client1.stop()
    await client2.stop()
    await client3.stop()
    await server.stop()
