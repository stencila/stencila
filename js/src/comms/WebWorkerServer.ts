import Server from './Server'

/**
 * A `Server` using the Web Workers API for communication.
 */
export default class WebWorkerServer extends Server {

  // Method overriden from `Server`

  start () {
    self.onmessage = (event: MessageEvent) => {
      const response = this.recieve(event.data)
      // @ts-ignore
      self.postMessage(response)
    }
  }

  stop () {
    self.onmessage = null
  }
}
