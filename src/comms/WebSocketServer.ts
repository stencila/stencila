import expressWs from 'express-ws'

import HttpServer from './HttpServer'

/**
 * A `Server` using WebSockets for communication.
 */
export default class WebSocketServer extends HttpServer {

  constructor () {
    super()

    expressWs(this.app)

    // @ts-ignore
    this.app.ws('/', (ws, req) => {
      ws.on('message', (request: string) => {
        ws.send(this.recieve(request))
      })
    })
  }
}
