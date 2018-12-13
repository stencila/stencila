import pytest

from stencilaschema.Processor import Processor
from stencilaschema.comms.TcpClient import TcpClient
from stencilaschema.comms.TcpServer import TcpServer
from stencilaschema.comms.UnixSocketClient import UnixSocketClient
from stencilaschema.comms.UnixSocketServer import UnixSocketServer
from stencilaschema.types.Thing import Thing
from stencilaschema.types.utils import dehydrate

processor = Processor()

string1 = '{"name":"thing1"}'
obj1 = {"name": "thing1"}
thing1 = Thing(name="thing1")

@pytest.mark.asyncio
async def test_start_stop():
    processor = Processor(
        server_types = [TcpServer, UnixSocketServer]
    )
    
    await processor.start()
    assert len(processor.servers) == 2
    keys = list(processor.servers.keys())
    assert keys[0][:6] == 'tcp://'
    assert keys[1][:7] == 'unix://'

    await processor.stop()
    assert len(processor.servers) == 0

@pytest.mark.asyncio
async def test_connect_disconnect():
    processor1 = Processor(
        server_types = [UnixSocketServer],
        client_types = [TcpClient]
    )
    processor2 = Processor(
        server_types = [TcpServer],
        client_types = [UnixSocketClient]
    )
    
    await processor1.start()
    await processor2.start()

    url1 = list(processor1.servers.values())[0].url
    url2 = list(processor2.servers.values())[0].url

    assert len(processor1.clients) == 0
    assert len(processor2.clients) == 0

    await processor1.connect(url2)
    await processor2.connect(url1)

    with pytest.raises(RuntimeError) as excinfo:
        await processor1.connect(url1)
        assert 'No client types able to connect to' in str(excinfo.value)

    assert len(processor1.clients) == 1
    assert isinstance(processor1.clients[url2], TcpClient)
    assert len(processor2.clients) == 1
    assert isinstance(processor2.clients[url1], UnixSocketClient)

    await processor1.disconnect(url2)
    await processor2.disconnect()

    assert len(processor1.clients) == 0
    assert len(processor2.clients) == 0

    await processor1.stop()
    await processor2.stop()

@pytest.mark.asyncio
async def test_discover():
    processor1 = Processor(
        server_types = [UnixSocketServer, TcpServer],
        client_types = [UnixSocketClient, TcpClient]
    )
    processor2 = Processor(
        server_types = [UnixSocketServer],
        client_types = [TcpClient]
    )
    processor3 = Processor(
        server_types = [TcpServer],
        client_types = [UnixSocketClient]
    )
    
    await processor1.start()
    await processor2.start()
    await processor3.start()

    url_1_unix = list(processor1.servers.keys())[0]
    url_1_tcp = list(processor1.servers.keys())[1]
    url_2_unix = list(processor2.servers.keys())[0]
    url_3_tcp = list(processor3.servers.keys())[0]

    await processor1.discover()
    await processor2.discover()
    await processor3.discover()

    assert len(processor1.clients) == 2
    assert isinstance(processor1.clients[url_2_unix], UnixSocketClient)
    assert isinstance(processor1.clients[url_3_tcp], TcpClient)

    assert len(processor2.clients) == 2
    assert isinstance(processor2.clients[url_1_tcp], TcpClient)
    assert isinstance(processor2.clients[url_3_tcp], TcpClient)

    assert len(processor3.clients) == 2
    assert isinstance(processor3.clients[url_1_unix], UnixSocketClient)
    assert isinstance(processor3.clients[url_2_unix], UnixSocketClient)
    
    # Discovering again shouldn't add any clients
    
    await processor1.discover()
    await processor2.discover()
    await processor3.discover()

    assert len(processor1.clients) == 2
    assert len(processor2.clients) == 2
    assert len(processor3.clients) == 2

    await processor1.stop()
    await processor2.stop()
    await processor3.stop()

@pytest.mark.asyncio
async def test_import():
    assert dehydrate(await processor.import_(string1)) == obj1
    assert dehydrate(await processor.import_(obj1)) == obj1
    assert dehydrate(await processor.import_(thing1)) == obj1

@pytest.mark.asyncio
async def test_export():
    assert await processor.export(thing1) == string1

@pytest.mark.asyncio
async def test_compile():
    assert dehydrate(await processor.compile(thing1)) == obj1

@pytest.mark.asyncio
async def test_build():
    assert dehydrate(await processor.build(thing1)) == obj1

@pytest.mark.asyncio
async def test_execute():
    assert dehydrate(await processor.execute(thing1)) == obj1
