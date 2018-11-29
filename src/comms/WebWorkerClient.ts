import Client from './Client'
import JsonRpcRequest from './JsonRpcRequest'

/**
 * A `Client` using the Web Workers API for communication.
 */
export default class WebWorkerClient extends Client {

  /**
   * A [`Worker`](https://developer.mozilla.org/en-US/docs/Web/API/Worker) instance.
   */
  worker: Worker

  /**
   * Constructor
   * 
   * @param server A URL to a Javascript file or a `Worker` instance
   */
  constructor (server: string | Worker) {
    super()

    if (typeof server === 'string') server = new Worker(server)

    this.worker = server
    this.worker.onmessage = (event: MessageEvent) => {
      this.recieve(event.data)
    }
  }

  // Overrides of `Client` methods

  send (request: JsonRpcRequest) {
    this.worker.postMessage(request)
  }
}
