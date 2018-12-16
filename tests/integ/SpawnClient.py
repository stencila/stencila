#!/usr/bin/env python3

# An integration tests for SpawnClient / SpawnServer

import asyncio
import os

from stencilaschema.comms.SpawnClient import SpawnClient

client = SpawnClient(['python3', os.path.join(os.path.dirname(__file__), 'SpawnServer.py')])

async def test():
    await client.start()

    thing1 = {"type": "Thing"}
    assert await client.execute(thing1) == thing1
    
    await client.stop()

loop = asyncio.get_event_loop()
loop.run_until_complete(test())
loop.close()
