import { Idiomorph } from 'idiomorph/dist/idiomorph.esm.js'

import { Entity } from '../nodes/entity'
import { NodeId } from '../types'
import { ChipToggleInterface } from '../ui/nodes/mixins/toggle-chip'
import { UIBaseClass } from '../ui/nodes/mixins/ui-base-class'
import { UINodeAuthors } from '../ui/nodes/properties/authors'

import { FormatPatch } from './format'

/**
 * A message received from VSCode by the web view
 */
type ReceivedMessage = DomPatchMessage | ViewNodeMessage

interface DomPatchMessage {
  type: 'dom-patch'
  patch: FormatPatch
}

interface ViewNodeMessage {
  type: 'view-node'
  nodeId: NodeId
  expandAuthors: boolean
}

/**
 * A message sent by the web view to VSCode
 *
 * TODO: This needs to be made consistent with the messages sent
 * by the other clients, in particular the WebSocket client.
 */
type SentMessage = {
  command: string
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [k: string]: any
}

interface VSCode {
  postMessage(message: SentMessage): void
}

/**
 * The VSCode API instance in the web view window
 *
 * Must be instantiated using `const vscode = acquireVsCodeApi()` in
 * the HTML of the view.
 */
declare const vscode: VSCode

/**
 * A client for sending and receiving messages to/from VSCode within a web view
 *
 * Note: this re-implements functionality in `FormatClient` and `DomClient` but
 * instead of using a Websocket, receives messages over VSCodes `postMessage`.
 */
export class WebViewClient {
  /**
   * The local version of the DOM HTML
   *
   * Used to check for missed patches and request a reset patch if necessary.
   */
  private version: number = 0

  /**
   * The DOM HTML string that is modified by patches and morphed onto `element`
   */
  private html: string

  /**
   * The HTML element that the document is rendered on
   */
  private renderRoot: HTMLElement

  constructor(renderRoot: HTMLElement) {
    this.version = 0
    this.html = ''
    this.renderRoot = renderRoot
    this.setWindowListener()
  }

  /**
   * Add event listeners to the window instance
   *
   * Note: any class methods used for/in the event callback must be bound to `this`
   * if they wish to use properties of `this`. Using arrow function
   * syntax when declaring methods will achieve this.
   */
  private setWindowListener() {
    //  Listener for 'message' events from the VSCode webview panel to the webview window
    window.addEventListener('message', (event) => this.receiveMessage(event))

    //  Listener for document commands from the view to send to the VSCode webview panel
    window.addEventListener('stencila-document-command', (event: CustomEvent) =>
      vscode.postMessage(event.detail)
    )
  }

  /**
   * Receive a 'message' event sent from VSCode to the web view `window`
   *
   * Note: must be an arrow function!
   *
   * @param event `Event` instance with `data` property carrying message
   */
  private receiveMessage({ data }: Event & { data: ReceivedMessage }) {
    const { type } = data

    switch (type) {
      case 'dom-patch':
        return this.handleDomPatchMessage(data)
      case 'view-node':
        return this.handleViewNodeMessage(data)
      default:
        throw new Error(`Unhandled received message type: ${type}`)
    }
  }

  /**
   * Handle a received `DomPatchMessage` message
   */
  private handleDomPatchMessage({ patch }: DomPatchMessage) {
    const { version, ops } = patch as unknown as FormatPatch

    // Is the patch a reset patch?
    const isReset = ops.length >= 1 && ops[0].type === 'reset'

    // Check for non-sequential patch and request a reset patch if necessary
    if (!isReset && version > this.version + 1) {
      // TODO: consider doing a reset here as is done in `./format.ts`
      // return
    }

    // Apply each operation in the patch
    for (const op of ops) {
      const { type, from, to, insert } = op

      if (type === 'reset' && insert !== undefined) {
        this.html = insert
      } else if (
        type === 'insert' &&
        typeof from === 'number' &&
        insert !== undefined
      ) {
        this.html = this.html.slice(0, from) + insert + this.html.slice(from)
      } else if (
        type === 'delete' &&
        typeof from === 'number' &&
        typeof to === 'number'
      ) {
        this.html = this.html.slice(0, from) + this.html.slice(to)
      } else if (
        type === 'replace' &&
        typeof from === 'number' &&
        typeof to === 'number' &&
        insert !== undefined
      ) {
        this.html = this.html.slice(0, from) + insert + this.html.slice(to)
      } else {
        console.error('Operation from server was not handled', op)
      }
    }

    // Update version
    this.version = version

    // Update element
    Idiomorph.morph(this.renderRoot.firstElementChild, this.html)
  }

  /**
   * Handle a received `ViewNodeMessage` message
   */
  private handleViewNodeMessage({ nodeId, expandAuthors }: ViewNodeMessage) {
    const targetEl = this.renderRoot.querySelector(`#${nodeId}`) as Entity
    if (targetEl) {
      targetEl.scrollIntoView({
        block: 'start',
        inline: 'nearest',
        behavior: 'smooth',
      })

      const card = targetEl.shadowRoot.querySelector(
        'stencila-ui-block-on-demand, stencila-ui-inline-on-demand'
      ) as UIBaseClass & ChipToggleInterface
      if (card) {
        card.openCard()
      }

      if (card && expandAuthors) {
        // Note that this querySelector call is intentionally on
        // the card itself and NOT its render root.
        const authors = card.querySelector(
          'stencila-ui-node-authors'
        ) as UINodeAuthors
        if (authors) {
          authors.expand()
        }
      }
    }
  }
}
