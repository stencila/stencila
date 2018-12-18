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

  // Additional methods for getting and posting to server

  /**
   * Make a GET request to the server
   *
   * @param path Path to request
   */
  async get (path: string) {
    return fetch(this.server + '/' + path)
  }

  /**
   * Make a POST request to the server
   *
   * @param path  Path to request
   * @param data Data to POST in the request body
   */
  async post (path: string, data?: {}) {
    return fetch(this.server + '/' + path, {
      method: 'POST',
      credentials: 'same-origin',
      headers: {
        'Content-Type': 'application/json; charset=utf-8',
        'Accept': 'application/json; charset=utf-8'
      },
      body: JSON.stringify(data)
    })
    .then(response => response.json())
  }
}
