import WebSocket from 'isomorphic-ws'

import Client from './Client'
import JsonRpcRequest from './JsonRpcRequest'

/**
 * A `Client` using the WebSockets API for communication.
 */
export default class WebSocketClient extends Client {

  /**
   * The URL of the `WebSocketServer` server
   */
  server: string

  /**
   * A [`WebSocket`](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket) instance
   */
  socket: WebSocket

  constructor (server: string) {
    super()
    this.server = server
    this.socket = new WebSocket(server)
    // @ts-ignore
    this.socket.addEventListener('message', (event: MessageEvent) => {
      this.recieve(event.data)
    })
  }

  // Overrides of `Client` methods

  send (request: JsonRpcRequest) {
    this.socket.send(JSON.stringify(request))
  }
}
