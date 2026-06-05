import { type DocumentId } from '../types'

const WEBSOCKET_CLOSING_READY_STATE = 2

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
   * Whether the client has been explicitly closed
   */
  private closed = false

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
   * Pending reconnect timer, if the socket has disconnected
   */
  private reconnectTimer?: ReturnType<typeof setTimeout>

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
    if (this.closed) {
      return
    }

    const protocol = window.location.protocol === 'http:' ? 'ws' : 'wss'
    const host = window.location.host
    const url = `${protocol}://${host}/~documents/${id}/websocket`

    this.ws = new WebSocket(url, subprotocol + '.stencila.org')

    const clientType = this.constructor.name.toLowerCase()

    this.ws.onopen = () => {
      console.debug(`🔌 ${this.constructor.name} connected`)

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

      this.handleConnected()
      this.hasConnected = true
    }

    this.ws.onclose = () => {
      if (this.closed) {
        return
      }

      console.debug(`🔌 ${this.constructor.name} disconnected`)

      document.body.classList.add(`${clientType}-disconnected`)

      window.dispatchEvent(new CustomEvent(`${clientType}-disconnected`))

      this.handleDisconnected()
      this.reconnectTimer = setTimeout(
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

      if (import.meta.env.DEV) {
        console.log(`🚩 ${this.constructor.name} received:`, message)
      }

      this.receiveMessage(message)
    }
  }

  /**
   * Close the WebSocket connection and stop reconnecting.
   */
  public close() {
    this.closed = true
    clearTimeout(this.reconnectTimer)
    this.reconnectTimer = undefined

    if (this.ws.readyState < WEBSOCKET_CLOSING_READY_STATE) {
      this.ws.close()
    }
  }

  /**
   * Called when the client's WebSocket connects.
   */
  protected handleConnected() {}

  /**
   * Called when the client's WebSocket disconnects.
   */
  protected handleDisconnected() {}

  /**
   * Receive a message from the server
   *
   * This method should be overridden by clients that need to
   * handle incoming messages from the server.
   *
   * @param message The message as a JavaScript object
   */
  // @ts-expect-error "function is a stub"
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  protected receiveMessage(message: Record<string, unknown>) {}

  /**
   * Send a message to the server
   *
   * @param message The message as a JavaScript object
   */
  protected sendMessage(message: Record<string, unknown>) {
    if (import.meta.env.DEV) {
      console.log(`📨 ${this.constructor.name} sending:`, message)
    }

    this.ws.send(JSON.stringify(message))
  }
}
