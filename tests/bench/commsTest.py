import asyncio
import os
import random
import string
import time

import pytest

from stencilaschema.Processor import Processor
from stencilaschema.comms.MmapClient import MmapClient
from stencilaschema.comms.MmapServer import MmapServer
from stencilaschema.comms.SpawnClient import SpawnClient
from stencilaschema.comms.TcpClient import TcpClient
from stencilaschema.comms.TcpServer import TcpServer
from stencilaschema.comms.UnixSocketClient import UnixSocketClient
from stencilaschema.comms.UnixSocketServer import UnixSocketServer

data = ''.join(random.choice(string.ascii_letters) for m in range(100))
small = {'foo': data}

async def run(client, thing):
    for i in range(10000):
        await client.execute(thing)

@pytest.mark.asyncio
async def test_mmap_small():
    server = MmapServer(Processor())
    await server.start()
    client = MmapClient(server.url)
    await client.start()

    start = time.perf_counter()
    await run(client, small)
    print(time.perf_counter()-start)

    await client.stop()
    await server.stop()

@pytest.mark.asyncio
async def test_spawn_small():
    client = SpawnClient(f'spawn://python3 {os.path.dirname(__file__)}/../comms/spawnServer.py')
    await client.start()

    start = time.perf_counter()
    await run(client, small)
    print(time.perf_counter()-start)

    await client.stop()

@pytest.mark.asyncio
async def test_uds_small():
    server = UnixSocketServer(Processor())
    await server.start()
    client = UnixSocketClient(server.url)
    await client.start()

    start = time.perf_counter()
    await run(client, small)
    print(time.perf_counter()-start)

    await client.stop()
    await server.stop()

@pytest.mark.asyncio
async def test_tcp_small():
    server = TcpServer(Processor())
    await server.start()
    client = TcpClient(server.url)
    await client.start()

    start = time.perf_counter()
    await run(client, small)
    print(time.perf_counter()-start)

    await client.stop()
    await server.stop()
