import { Entity } from '../nodes/entity'
import { NodeId } from '../types'
import { ChipToggleInterface } from '../ui/nodes/mixins/toggle-chip'
import { UIBaseClass } from '../ui/nodes/mixins/ui-base-class'
import { UINodeAuthors } from '../ui/nodes/properties/authors'

/**
 * A message received from VSCode by the web view
 */
type ReceivedMessage = ViewNodeMessage

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
 */
export class WebViewClient {
  constructor(element: HTMLElement) {
    this.element = element
    this.setWindowListener()
  }

  private element: HTMLElement

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
      case 'view-node':
        return this.handleViewNodeMessage(data)
      default:
        throw new Error(`Unhandled received message type: ${type}`)
    }
  }

  /**
   * Handle a received `ViewNodeMessage` message
   */
  private handleViewNodeMessage({ nodeId, expandAuthors }: ViewNodeMessage) {
    const targetEl = this.element.querySelector(`#${nodeId}`) as Entity
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
