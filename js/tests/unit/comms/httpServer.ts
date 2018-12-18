#!/usr/bin/env node

import HttpServer from '../../../src/comms/HttpServer'

const server = new HttpServer(undefined, 0)
server.run()
