import expressWs from 'express-ws'

import HttpServer from './HttpServer'
import Processor from '../Processor'

/**
 * A `Server` using WebSockets for communication.
 */
export default class WebSocketServer extends HttpServer {

  constructor (processor?: Processor, logging?: number, port?: number, address?: string) {
    super(processor, logging, port, address)

    expressWs(this.app)

    // @ts-ignore
    this.app.ws('/', (ws, req) => {
      ws.on('message', (request: string) => {
        ws.send(this.recieve(request))
      })
    })
  }
}
