#!/usr/bin/env python3

# An integration tests for StdioClient / StdioServer

import asyncio
import os

from stencilaschema.comms.StdioClient import StdioClient

client = StdioClient(['python3', os.path.join(os.path.dirname(__file__), 'StdioServer.py')])

async def test():
    await client.start()

    thing1 = {"type": "Thing"}
    assert await client.execute(thing1) == thing1
    
    await client.stop()

loop = asyncio.get_event_loop()
loop.run_until_complete(test())
loop.close()
