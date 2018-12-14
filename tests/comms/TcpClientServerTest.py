import pytest

from stencilaschema.comms.JsonEncoder import JsonEncoder
from stencilaschema.comms.JsonGzipBase64Encoder import JsonGzipBase64Encoder
from stencilaschema.comms.TcpClient import TcpClient
from stencilaschema.comms.TcpServer import TcpServer

from helpers.processors import TestProcessor

@pytest.mark.asyncio
async def test_client_server():
    # Create test processor
    processor = TestProcessor()

    # Start the server and several clients
    server = TcpServer(processor, encoders=[JsonEncoder(), JsonGzipBase64Encoder()])
    await server.start()
    client1 = TcpClient(server.url)
    await client1.start()
    client2 = TcpClient(server.url, encoders=[JsonGzipBase64Encoder(), JsonEncoder()])
    await client2.start()
    client3 = TcpClient(server.url, encoders=[JsonGzipBase64Encoder()])
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
