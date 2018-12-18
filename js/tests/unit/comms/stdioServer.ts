#!/usr/bin/env node

import StdioServer from '../../../src/comms/StdioServer'

const server = new StdioServer()
server.run()
