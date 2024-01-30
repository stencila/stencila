import { type DocumentId } from '../types'

/**
 * The abstract base class for all clients
 *
 * Sets the following classes on `document.body` where `${clientType}` is the
 * lowercase name of the client class:
 *
 * - ${clientType}-connected
 * - ${clientType}-disconnected
 *
 * Note that these are not mutually exclusive and if both are present it indicates
 * that the client was connected but got disconnected and is now attempting
 * to reconnect.
 *
 * Emits the following events on `window`:
 *
 * - ${clientType}-connected
 * - ${clientType}-disconnected
 * - ${clientType}-reconnected
 */
export abstract class Client {
  /**
   * The client's WebSocket connection
   */
  private ws: WebSocket

  /**
   * Status of the websocket connection
   */
  private hasConnected: boolean = false

  /**
   * Initial reconnection interval
   */
  private initialReconnectInterval: number = 1000

  /**
   * Current reconnection interval
   */
  private currentReconnectInterval: number = this.initialReconnectInterval

  /**
   * Construct a new document client
   *
   * @param id  The id of the document
   * @param subprotocol The WebSocket subprotocol to use
   */
  constructor(id: DocumentId, subprotocol: string) {
    this.connect(id, subprotocol)
  }

  /**
   * Create a new WebSocket and assign methods to it to handle `onopen` etc events.
   *
   * @param id  The id of the document
   * @param subprotocol The WebSocket subprotocol to use
   */
  private connect(id: DocumentId, subprotocol: string) {
    const protocol = window.location.protocol === 'http:' ? 'ws' : 'wss'
    const host = window.location.host
    const url = `${protocol}://${host}/~ws/${id}`

    this.ws = new WebSocket(url, subprotocol + '.stencila.org')

    const clientType = this.constructor.name.toLowerCase()

    this.ws.onopen = () => {
      console.debug(`${clientType} websocket open`)

      const classList = document.body.classList
      classList.add(`${clientType}-connected`)
      classList.remove(`${clientType}-disconnected`)

      window.dispatchEvent(
        new CustomEvent(
          this.hasConnected
            ? `${clientType}-reconnected`
            : `${clientType}-connected`
        )
      )

      this.hasConnected = true
    }

    this.ws.onclose = () => {
      console.debug(`${clientType} websocket closed`)

      document.body.classList.add(`${clientType}-disconnected`)

      window.dispatchEvent(new CustomEvent(`${clientType}-disconnected`))

      setTimeout(
        () => {
          if (this.currentReconnectInterval < 120000) {
            this.currentReconnectInterval *= 1.5
          }
          this.connect(id, subprotocol)
        },
        this.currentReconnectInterval + Math.random() * 3000
      )
    }

    this.ws.onmessage = (event: MessageEvent<string>) => {
      const message = JSON.parse(event.data)

      if (process.env.NODE_ENV === 'development') {
        console.log(`🚩 ${this.constructor.name} received:`, message)
      }

      this.receiveMessage(message)
    }
  }

  /**
   * Receive a message from the server
   *
   * This method should be overridden by clients that need to
   * handle incoming messages from the server.
   *
   * @param message The message as a JavaScript object
   */
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  protected receiveMessage(message: Record<string, unknown>) {}

  /**
   * Send a message to the server
   *
   * @param message The message as a JavaScript object
   */
  protected sendMessage(message: Record<string, unknown>) {
    if (process.env.NODE_ENV === 'development') {
      console.log(`📨 ${this.constructor.name} sending:`, message)
    }

    this.ws.send(JSON.stringify(message))
  }
}
