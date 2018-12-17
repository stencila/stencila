import os
import pytest

from stencilaschema.comms.SpawnClient import SpawnClient

from helpers.processors import PersonProcessor

@pytest.mark.skip
@pytest.mark.asyncio
async def test_client_server():

    client = SpawnClient(f'spawn://python3 {os.path.dirname(__file__)}/spawnServer.py')
    assert client.url[:8] == 'spawn://'
    
    await client.start()

    pre = {'type': 'Person', 'givenNames': ['Peter'], 'familyNames': ['Pan']}
    post = {'type': 'Person', 'givenNames': ['Peter'], 'familyNames': ['Pan'], 'name': 'Peter Pan'}
    assert await client.execute(pre) == post

    await client.stop()
