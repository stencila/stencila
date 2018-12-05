import pytest

from stencilaschema.comms.StdioClient import StdioClient
from stencilaschema.comms.StdioServer import StdioServer

def test_client():
    client = StdioClient([])
    assert client.url == 'pipe://stdio'

@pytest.mark.asyncio
async def test_server(capsys):
    # Disable stdout/stderr/stdin capturing
    with capsys.disabled():
        server = StdioServer()
        assert server.url == 'pipe://stdio'
        await server.start()
        
        # TODO This fails:
        # await server.stop()

@pytest.mark.asyncio
async def test_client_server(capsys):
    # Disable stdout/stderr/stdin capturing
    with capsys.disabled():
        # Test that the client can start a peer server
        client = StdioClient(['python3', 'tests/comms/stdioServer.py'])
        
        # TODO This hangs:
        # await client.start()
        
        #await client.execute({"type": "Thing"})
        
        # TODO This fails:
        # await client.stop()
