import asyncio
import os
import tempfile

import pytest

from stencilaschema.comms.MmapClient import MmapClient
from stencilaschema.comms.MmapServer import MmapServer

from helpers.processors import PersonProcessor

@pytest.mark.asyncio
async def test_server():
    processor = PersonProcessor()
    server = MmapServer(processor)
    await server.start()

    assert server._id
    assert server._path

    await server.stop()

@pytest.mark.asyncio
async def test_client_server():
    processor = PersonProcessor()
    server = MmapServer(processor)
    await server.start()
    
    client = MmapClient(server.url)
    await client.start()

    assert server.url[:7] == 'mmap://'
    assert client.url == server.url

    pre = {'type': 'Person', 'givenNames': ['Peter'], 'familyNames': ['Pan']}
    post = {'type': 'Person', 'givenNames': ['Peter'], 'familyNames': ['Pan'], 'name': 'Peter Pan'}
    assert await client.execute(pre) == post

    await client.stop()
    await server.stop()
