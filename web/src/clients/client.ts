import { type DocumentId } from '../types'

/**
 * The abstract base class for all clients
 *
 * TODO: Implement WebSocket connection state and reconnection logic
 * https://github.com/stencila/stencila/issues/1785
 */
export abstract class Client {
  /**
   * The client's WebSocket connection
   */
  private ws: WebSocket

  /**
   * name of the context the client object is used
   */
  private clientType: string

  /**
   * status of the websocket connection
   */
  private hasConnected: boolean = false

  /**
   * initial interval timeout
   */
  private initialReconectInterval: number = 1000

  /**
   * current interval timeout
   */
  private currentReconnectInterval: number = this.initialReconectInterval

  /**
   * Construct a new document client
   *
   * @param id  The id of the document
   * @param subprotocol The WebSocket subprotocol to use
   */
  constructor(id: DocumentId, subprotocol: string, clientType?: string) {
    const protocol = window.location.protocol === 'http:' ? 'ws' : 'wss'
    const host = window.location.host

    this.clientType = clientType ?? 'unknown'

    this.connect(`${protocol}://${host}/~ws/${id}`, subprotocol)
  }

  /**
   * Create and connect new websocket, and assign methods.
   *
   * @param url
   * @param subprotocol
   */
  private connect(url, subprotocol) {
    this.ws = new WebSocket(url, subprotocol + '.stencila.org')

    this.ws.onopen = () => {
      console.debug(`${this.clientType}-client websocket open`)

      const classList = document.body.classList
      classList.add(`${this.clientType}-client-connected`)
      classList.remove(`${this.clientType}-client-disconnected`)

      window.dispatchEvent(
        new CustomEvent(
          this.hasConnected
            ? `${this.clientType}-ws-reconnect`
            : `${this.clientType}-ws-connect`
        )
      )

      this.hasConnected = true
    }

    this.ws.onclose = () => {
      console.debug(`${this.clientType}-client on closed`)

      document.body.classList.add(`${this.clientType}-client-disconnected`)

      window.dispatchEvent(new CustomEvent(`${this.clientType}-ws-disconnect`))

      setTimeout(
        () => {
          if (this.currentReconnectInterval < 120000) {
            this.currentReconnectInterval *= 1.5
          }
          this.connect(url, subprotocol)
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
