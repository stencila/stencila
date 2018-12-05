import pytest

from stencilaschema.comms.Client import Client
from stencilaschema.comms.jsonRpc import Request, Response

@pytest.mark.asyncio
async def test_recieve():
    client = Client()

    async def write(message):
        pass
    client.write = write

    future = await client.send(Request(method="compile", id=1))
    client.recieve(Response(id=1, result={"type": "Thing"}))
    response = await future
    assert response.result == {"type": "Thing"}
