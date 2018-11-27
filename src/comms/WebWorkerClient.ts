import Client from './Client'
import JsonRpcRequest from './JsonRpcRequest'

/**
 * A `Client` using the Web Workers API for communication.
 */
export default class WebWorkerClient extends Client {

  /**
   * A [`Worker`](https://developer.mozilla.org/en-US/docs/Web/API/Worker) instance,
   * usually the `worker` property of a `WebWorkerServer`.
   */
  worker: Worker

  constructor (worker: Worker) {
    super()

    this.worker = worker
    this.worker.onmessage = (event: MessageEvent) => {
      this.recieve(event.data)
    }
  }

  // Overrides of `Client` methods

  send (request: JsonRpcRequest) {
    this.worker.postMessage(request)
  }
}
