import { customAlphabet } from 'nanoid'
import { Client as WsClient } from 'rpc-websockets'

import { notify } from '../components/base/alert'
import { currentMode, Mode } from '../mode'
import { panic } from '../patches/checks'
import { applyPatch } from '../patches/dom'
import { Document, DocumentConfig, DocumentPatchEvent, Patch } from '../types'

export type DocumentTopic = 'patched'

type ClientConnectOptions = ConstructorParameters<typeof WsClient>[1]

export enum LogLevel {
  Debug = 3,
  Info = 2,
  Warn = 1,
  Error = 0,
}

const localStorageKeys = {
  browserId: 'StencilaDocumentClient.browserId',
  logLevel: 'StencilaDocumentClient.logLevel',
}

/**
 * A browser-based client for communicating with a document instance on a Stencila document server
 */
export class DocumentClient {
  origin: string

  /**
   * A unique identifier for this client
   *
   * This will be different in each browser tab and will change on page refresh.
   * Used to resolve which document instance this client should be connected to,
   * for example on a reconnect.
   */
  private clientId: string

  /**
   * A unique identifier for this browser
   *
   * The id is stored in `localStorage` and retrieved for each new connection.
   * Used to resolve which document instance this client should be connected to:
   * e.g. if there is no instance linked to the `clientId` but there is one for the
   * `browserId`. Intended to avoid multiple forks of a document for the same browser.
   */
  private browserId: string

  /**
   * The id of the document instance that this client is connected to
   */
  private documentId: string

  /**
   * The version of the document instance
   *
   * Used to
   */
  private documentVersion: number

  /**
   * The URL used to establish a connection for the document
   *
   * Created in constructor from `origin` and `path`, with `clientId` and
   * `browserId` added as query parameters.
   */
  private connectionUrl: string

  /**
   * A WebSocket JSON-RPC client for the document
   *
   * Created in constructor.
   */
  private wsClient?: WsClient

  private wsClientError?: string

  logLevel: LogLevel = LogLevel.Info

  /**
   * Construct a document client
   *
   * If no config is supplied, then the `window.stencilaConfig` object will be used.
   * If that is missing, or has missing properties, those will be inferred from `window.location`.
   */
  constructor(config?: DocumentConfig) {
    // Generate the client id
    this.clientId = this.generateId('cl')

    // Retrieve or generate/store the browser id
    let browserId = window.localStorage.getItem(localStorageKeys.browserId)
    if (browserId === null) {
      browserId = this.generateId('br')
      window.localStorage.setItem(localStorageKeys.browserId, browserId)
    }
    this.browserId = browserId

    const { origin, path, token, documentId } = config ?? {}

    // If arguments not supplied fallback to constructing them from the path or
    // server supplied config
    this.origin =
      typeof origin === 'string'
        ? origin
        : `${window.location.protocol.replace('http', 'ws')}//${
            window.location.host
          }`
    const path_ = typeof path === 'string' ? path : window.location.pathname
    const token_ =
      typeof token === 'string' ? token : window.stencilaConfig.token

    let connectionUrl = `${this.origin}/~rpc${path_}?client=${this.clientId}&browser=${this.browserId}`
    if (typeof token === 'string' && token.length > 0) {
      connectionUrl += `&token=${token}`
    }

    this.connectionUrl = connectionUrl
    this.documentId = documentId ?? ''

    // Initialize log level from local storage if possible
    const logLevel = window.localStorage.getItem(localStorageKeys.logLevel)
    if (logLevel !== null) {
      this.logLevel = new Number(logLevel) as LogLevel
    }

    // Listen for within browser events and forward on to the methods below

    window.addEventListener('stencila-document-patch', (event) => {
      const {
        detail: { patch },
      } = event as DocumentPatchEvent
      this.sendPatch(patch)
    })

    window.addEventListener(
      'stencila-document-compile',
      (event: CustomEvent) => {
        const {
          detail: { nodeId },
        } = event
        this.compile(nodeId)
      }
    )

    window.addEventListener(
      'stencila-document-execute',
      (event: CustomEvent) => {
        const {
          detail: { nodeId, ordering },
        } = event
        this.execute(nodeId, ordering)
      }
    )

    window.addEventListener('stencila-document-write', (event: CustomEvent) => {
      const {
        detail: { nodeId },
      } = event
      this.write(nodeId)
    })
  }

  /**
   * Generate a unique id for client or browser
   *
   * @param prefix The `cl` or `br` prefix
   */
  generateId(prefix: string): string {
    return (
      prefix +
      '_' +
      customAlphabet(
        '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz',
        30
      )()
    )
  }

  // Logging functions

  debug(message: string) {
    if (this.logLevel >= LogLevel.Debug) notify(message, 'neutral', 'bug')
  }

  info(message: string) {
    if (this.logLevel >= LogLevel.Info)
      notify(message, 'primary', 'info-circle')
  }

  warn(message: string) {
    if (this.logLevel >= LogLevel.Warn)
      notify(message, 'warning', 'exclamation-triangle')
  }

  error(message: string) {
    if (this.logLevel >= LogLevel.Error)
      notify(message, 'danger', 'exclamation-octagon')
  }

  /**
   * Change the log level of the client
   *
   * Stores the setting to the browser's local storage.
   */
  changeLogLevel(level: LogLevel) {
    this.logLevel = level
    window.localStorage.setItem(localStorageKeys.logLevel, level.toString())
  }

  /**
   * Connect to the document server
   *
   * @param clientOptions Options to pass to the WebSocket RPC client
   */
  async connect(clientOptions: ClientConnectOptions = {}) {
    this.wsClient = new WsClient(this.connectionUrl, {
      reconnect_interval: 1000 + 3000 * Math.random(), // random interval between 1 and 4 seconds
      max_reconnects: 1000,
      ...clientOptions,
    })

    // Wait for WebSocket client to connect
    this.emit('stencila-client-connecting')
    this.wsClientError = 'Connecting'
    await new Promise<void>((resolve) =>
      this.wsClient?.on('open', () => resolve())
    )
    this.wsClientError = undefined

    // If the client closes without an explicit disconnect...
    this.wsClient?.on('error', (error) => {
      if (this.wsClientError === undefined) {
        this.wsClientError = error.message ?? 'Connection error'
        this.emit('stencila-client-disconnected')
        this.error('Lost connection to server. Trying to reconnect...')
      }
    })

    // If the client reopens recover from error
    this.wsClient?.on('open', (error) => {
      this.wsClientError = undefined
      this.emit('stencila-client-connected')
      this.debug('Successfully reconnected to server')
    })

    // Emit event and logs
    this.emit('stencila-client-connected')
    this.debug(`Successfully connected to server: ${this.origin}`)

    // Subscribe to patches depending upon mode
    if (currentMode() >= Mode.Dynamic) {
      this.subscribe('patched', (patch) => this.receivePatch(patch as Patch))
    }
  }

  /**
   * Disconnect from the document server
   */
  disconnect() {
    if (this.wsClientError === undefined) {
      this.wsClient?.close()
    }
    this.wsClient = undefined

    // Emit event and logs
    this.emit('stencila-client-disconnected')
    this.debug(`Successfully disconnected from server: ${this.origin}`)
  }

  /**
   * Get the connection status of the client
   */
  status(): 'disconnected' | 'connecting' | 'connected' | 'reconnecting' {
    if (this.wsClientError === 'Connecting') {
      return 'connecting'
    }
    if (this.wsClient === undefined) {
      return 'disconnected'
    } else if (this.wsClientError !== undefined) {
      return 'reconnecting'
    } else {
      return 'connected'
    }
  }

  /**
   * Make a WebSocket JSON-RPC call
   */
  call(method: string, params: Record<string, unknown>): Promise<unknown> {
    if (!this.wsClient) {
      this.error(`Not yet connected to server`)
      return Promise.resolve()
    }
    return this.wsClient
      .call(method, { ...params, documentId: this.documentId })
      .catch((error) => {
        this.error(`Error when making request to server: ${error.message}`)
      })
  }

  /**
   * Load content into the document
   *
   * If `format` is not supplied, the content will be assumed to be the current
   * format of the document e.g. `md`. If `content` is not a string and format
   * is 'json' of undefined, then `content` will be stringify to JSON.
   */
  async load(content: unknown, format?: string): Promise<void> {
    if (
      typeof content !== 'string' &&
      (format === 'json' || format === undefined)
    ) {
      content = JSON.stringify(content)
      format = 'json'
    }
    return this.call('documents.load', {
      content,
      format,
    }) as Promise<void>
  }

  /**
   * Dump all, or part, of the document to a string
   *
   * If `format` is not supplied, the content will be assumed to be the current
   * format of the document e.g. `md`. If `nodeId` is supplied then only that
   * node will be dumped.
   */
  async dump(format?: string, nodeId?: string): Promise<string> {
    return this.call('documents.dump', {
      format,
      nodeId,
    }) as Promise<string>
  }

  /**
   * Subscribe to a document topic
   */
  async subscribe(
    topic: DocumentTopic,
    handler: (detail: unknown) => void
  ): Promise<Document> {
    this.wsClient?.on(`documents:${this.documentId}:${topic}`, handler)
    return this.call('documents.subscribe', {
      topic,
    }) as Promise<Document>
  }

  /**
   * Unsubscribe from a document topic
   */
  async unsubscribe(topic: DocumentTopic): Promise<Document> {
    this.wsClient?.off(`documents:${this.documentId}:${topic}`)
    return this.call('documents.unsubscribe', {
      topic,
    }) as Promise<Document>
  }

  /**
   * Send a document patch to the server
   *
   * Will generate an error if the patch could not be applied e.g. no node with the id could
   * be found or the patch was inconsistent with the node.
   *
   * Previously we allowed for a `then` parameter to be passed here.
   * That is now done on server side so that they can be a consistent (but user configurable)
   * approach to what happens when different node types are patched (rather that having to be
   * reimplemented for different clients).
   */
  async sendPatch(patch: Patch): Promise<void> {
    // During development it's very useful to see the patch operations being sent
    if (process.env.NODE_ENV !== 'production') {
      const { target, address, ops } = patch
      console.log(
        '📢 Sending patch:',
        JSON.stringify({ actor: this.clientId, target, address })
      )
      for (const op of ops) console.log('  ', JSON.stringify(op))
    }

    return this.call('documents.patch', {
      patch: { actor: this.clientId, ...patch },
    }) as Promise<void>
  }

  // Receive a document patch from the server
  receivePatch(
    patch: Patch,
    callback: (patch: Patch) => void = applyPatch
  ): void {
    const { version, actor, target, ops } = patch

    // Check the patch version number and panic if out of order
    const lastVersion = this.documentVersion
    if (version !== undefined && lastVersion !== undefined) {
      if (version !== lastVersion + 1) {
        throw panic(`Expected patch version ${lastVersion + 1}, got ${version}`)
      }
    }

    // Update the client's patch version
    if (version !== undefined) {
      this.documentVersion = version
    }

    // Ignore any patches where this client was the actor
    if (actor === this.clientId) return

    // During development it's useful to see which patches are being received
    if (process.env.NODE_ENV !== 'production') {
      console.log(
        '📩 Received patch:',
        JSON.stringify({ version, actor, target })
      )
      for (const op of ops) console.log('  ', JSON.stringify(op))
    }

    callback(patch)
  }

  /**
   * Compile a node or multiple nodes
   */
  async compile(nodeId: null | string): Promise<void> {
    if (process.env.NODE_ENV !== 'production') {
      console.log('📘 Compiling node:', nodeId)
    }

    return this.call('documents.compile', {
      nodeId,
    }) as Promise<void>
  }

  /**
   * Execute a node or multiple nodes
   */
  async execute(
    nodeId: null | string,
    ordering: 'Single' | 'Appearance' | 'Topological' = 'Topological'
  ): Promise<void> {
    if (process.env.NODE_ENV !== 'production') {
      console.log('🏃 Executing node:', nodeId, ordering)
    }

    return this.call('documents.execute', {
      nodeId,
      ordering,
    }) as Promise<void>
  }

  /**
   * Cancel execution of a node or multiple nodes
   */
  async cancel(
    nodeId: null | string,
    scope: 'Single' | 'All' = 'All'
  ): Promise<void> {
    if (process.env.NODE_ENV !== 'production') {
      console.log('🔇 Cancelling execution of node:', nodeId, scope)
    }

    return this.call('documents.cancel', {
      nodeId,
      scope,
    }) as Promise<void>
  }

  /**
   * Write the document
   */
  async write(nodeId: null | string): Promise<void> {
    if (process.env.NODE_ENV !== 'production') {
      console.log('💾 Writing document')
    }

    return this.call('documents.write', {
      nodeId,
    }) as Promise<void>
  }

  /**
   * Restart one, or all, of the kernels in a document's kernel space
   *
   * If `kernelId` is not supplied then all kernels in the kernel space
   * will be restarted.
   */
  async restart(kernelId?: string): Promise<void> {
    return this.call('documents.restart', {
      kernelId,
    }) as Promise<void>
  }

  /**
   * Get a list of kernels in a document's kernel space
   */
  async kernels(): Promise<void> {
    return this.call('documents.kernels', {}) as Promise<void>
  }

  /**
   * Get a list of symbols in a document's kernel space
   */
  async symbols(): Promise<void> {
    return this.call('documents.symbols', {}) as Promise<void>
  }

  /**
   * Emit a custom event on the window
   *
   * @param name The name of the custom event
   * @param detail The event details
   * @param options Options for the custom event
   * @returns CustomEvent
   */
  emit(name: string, detail = {}, options?: CustomEventInit) {
    const event = new CustomEvent(name, {
      detail,
      bubbles: true,
      cancelable: false,
      composed: true,
      ...options,
    })
    window.dispatchEvent(event)
  }
}
