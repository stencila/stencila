#!/usr/bin/env python3

import asyncio

from stencilaschema.comms.StdioServer import StdioServer

server = StdioServer()

if __name__ == '__main__':
    loop = asyncio.get_event_loop()
    loop.run_until_complete(server.start())
    loop.close()
