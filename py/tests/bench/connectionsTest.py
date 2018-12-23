import asyncio
import math
import os

import pytest

from stencilaschema.comms.MmapConnection import MmapConnection
from stencilaschema.comms.StreamConnection2 import StreamConnection

protocols = {}

# Mmap
def setup_mmap():
    remote = MmapConnection()
    local = MmapConnection(remote.path)
    protocols['mmap'] = [local, remote]
setup_mmap()

# Pipes
async def setup_pipe():
    protocols['pipe'] = [None, None]
    if not os.path.exists('/tmp/pipe1'):
        os.mkfifo('/tmp/pipe1')
    if not os.path.exists('/tmp/pipe1'):
        os.mkfifo('/tmp/pipe2')
    input1 = os.fdopen(os.open('/tmp/pipe1', os.O_RDONLY | os.O_NONBLOCK))
    input2 = os.fdopen(os.open('/tmp/pipe2', os.O_RDONLY | os.O_NONBLOCK))
    output1 = os.fdopen(os.open('/tmp/pipe2', os.O_WRONLY | os.O_NONBLOCK))
    output2 = os.fdopen(os.open('/tmp/pipe1', os.O_WRONLY | os.O_NONBLOCK))
    protocols['pipe'][0] = await StreamConnection.from_files(input1, output1)
    protocols['pipe'][1] = await StreamConnection.from_files(input2, output2)
loop = asyncio.get_event_loop()
loop.run_until_complete(setup_pipe())

# UDS
async def setup_unix():
    protocols['unix'] = [None, None]
    def connected_callback(reader, writer):
        protocols['unix'][1] = StreamConnection(reader, writer)
    await asyncio.start_unix_server(connected_callback, '/tmp/unix.socket')
    reader, writer = await asyncio.open_unix_connection('/tmp/unix.socket')
    protocols['unix'][0] = StreamConnection(reader, writer)
loop = asyncio.get_event_loop()
loop.run_until_complete(setup_unix())

# TCP
async def setup_tcp():
    protocols['tcp'] = [None, None]
    def connected_callback(reader, writer):
        protocols['tcp'][1] = StreamConnection(reader, writer)
    await asyncio.start_server(connected_callback, '127.0.0.1', 2345)
    reader, writer = await asyncio.open_connection('127.0.0.1', 2345)
    protocols['tcp'][0] = StreamConnection(reader, writer)
loop = asyncio.get_event_loop()
loop.run_until_complete(setup_tcp())


async def roundtrip(local, remote, message):
    await local.write(message)
    received = await remote.read()
    await remote.write(received)
    echo = await local.read()
    assert len(echo) == len(message)

def run(local, remote, payload):
    async def repeat():
        await roundtrip(local, remote, payload)
    loop = asyncio.get_event_loop()
    loop.run_until_complete(repeat())

@pytest.mark.parametrize('protocol', ['mmap', 'pipe', 'unix', 'tcp'])
@pytest.mark.parametrize('size', range(2, 25, 2))
def test(benchmark, protocol, size):
    # Group tests by message size
    benchmark.group = f'{size:03d}'
    local, remote = protocols[protocol]
    messge = b'\0' * int(math.pow(2, size))
    benchmark(run, local, remote, messge)
