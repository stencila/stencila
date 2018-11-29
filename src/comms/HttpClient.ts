import fetch from 'cross-fetch'

import Client from './Client'
import JsonRpcRequest from './JsonRpcRequest'

/**
 * A `Client` using HTTP/S for communication.
 */
export default class HttpClient extends Client {

  /**
   * The URL of the server
   */
  server: string

  constructor (server: string) {
    super()
    this.server = server
  }

  // Overrides of `Client` methods

  async send (request: JsonRpcRequest) {
    return fetch(this.server, {
      method: 'POST',
      mode: 'cors', // no-cors, cors, *same-origin
      cache: 'no-cache', // *default, no-cache, reload, force-cache, only-if-cached
      credentials: 'same-origin', // include, *same-origin, omit
      headers: {
        'Content-Type': 'application/json; charset=utf-8',
        'Accept': 'application/json; charset=utf-8'
      },
      body: JSON.stringify(request)
    })
    .then(response => response.json())
    .then(response => this.recieve(response))
  }
}
