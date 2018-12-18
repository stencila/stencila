#!/usr/bin/env node

import WebSocketServer from '../../../src/comms/WebSocketServer'

const server = new WebSocketServer()
server.run()
