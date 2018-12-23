import pytest

from stencilaschema.comms.MmapConnection import MmapConnection

@pytest.mark.asyncio
async def test_client_server():
    conn1 = MmapConnection()
    conn2 = MmapConnection(conn1.path)

    message = 'hello'.encode()
    await conn1.write(message)
    assert await conn2.read() == message

    message = 'world'.encode()
    await conn2.write(message)
    assert await conn1.read() == message
