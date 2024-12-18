import { Idiomorph } from 'idiomorph/dist/idiomorph.esm.js'

import { Entity } from '../nodes/entity'
import { NodeId } from '../types'
import { MarkerToggleInterface } from '../ui/nodes/mixins/toggle-marker'
import { UIBaseClass } from '../ui/nodes/mixins/ui-base-class'
import { UINodeAuthors } from '../ui/nodes/properties/authors'

import { FormatPatch } from './format'

/**
 * A message received from VSCode by the web view
 */
type ReceivedMessage = DomPatchMessage | ViewNodeMessage | ScrollSyncMessage

interface DomPatchMessage {
  type: 'dom-patch'
  patch: FormatPatch
}

interface ViewNodeMessage {
  type: 'view-node'
  nodeId: NodeId
  expandAuthors: boolean
}

interface ScrollSyncMessage {
  type: 'scroll-sync'
  startId?: string
  endId?: string
  cursorId?: string
}

/**
 * A message sent by the web view to VSCode
 *
 * TODO: This needs to be made consistent with the messages sent
 * by the other clients, in particular the WebSocket client.
 */
type SentMessage = DomResetMessage | CommandMessage | ScrollSyncMessage

interface DomResetMessage {
  type: 'dom-reset'
}

interface CommandMessage {
  type: 'command'
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
export class PreviewClient {
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
   * The render root of the view
   *
   * This will contain an element for the document root which is morphed by patches.
   */
  private renderRoot: HTMLElement

  /**
   * A count of the number of failed DOM morphing attempts
   *
   * If the second, then the root element will just be
   * set to the HTML of the reset patch, with no attempt to morph.
   */
  private failedMorph: number = 0

  /**
   * The id of the top node in the viewport
   *
   * Used for syncing scrolling back to the VSCode editor
   */
  private startId?: NodeId

  /**
   * The id of the bottom node in the viewport
   *
   * Used for syncing scrolling back to the VSCode editor
   */
  private endId?: NodeId

  /**
   * Intersection observer used to update `startId` and `endId`
   */
  private intersectionObserver: IntersectionObserver

  /**
   * Mutation observer to add and remove elements from the intersection observer
   */
  private mutationObserver: MutationObserver

  constructor(rootElement: HTMLElement) {
    this.version = 0
    this.html = ''
    this.renderRoot = rootElement
    this.failedMorph = 0
    this.setWindowListener()
    this.setupObservers()
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
      vscode.postMessage({ type: 'command', ...event.detail })
    )
  }

  /**
   * Setup intersection and mutation observers
   */
  private setupObservers() {
    // Create intersection observer for scroll sync
    this.intersectionObserver = new IntersectionObserver((entries) => {
      // Get all elements that are currently intersecting
      const intersecting = entries
        .filter((entry) => entry.isIntersecting)
        .map((entry) => ({
          id: entry.target.id,
          topDistance: entry.boundingClientRect.top,
          bottomDistance: entry.boundingClientRect.bottom,
        }))

      // Get the ids of the elements closest to the top and bottom
      const startId = intersecting.sort(
        (a, b) => Math.abs(a.topDistance) - Math.abs(b.topDistance)
      )[0]?.id
      const endId = intersecting.sort(
        (a, b) => Math.abs(a.bottomDistance) - Math.abs(b.bottomDistance)
      )[0]?.id

      // Only send update if the top or bottom ids have changed
      let update = false
      if (startId && startId !== this.startId) {
        this.startId = startId
        update = true
      }
      if (endId && endId !== this.endId) {
        this.endId = endId
        update = true
      }
      if (update) {
        vscode.postMessage({
          type: 'scroll-sync',
          startId: this.startId,
          endId: this.endId,
        })
      }
    })

    // Create mutation observer to watch for new elements
    this.mutationObserver = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        if (mutation.type === 'childList') {
          // Handle added nodes
          mutation.addedNodes.forEach((node) => {
            if (node.nodeType === Node.ELEMENT_NODE) {
              const elem = node as Element
              // Observe the new element if it has an ID
              if (elem.id) {
                this.intersectionObserver.observe(elem)
              }
              // Also check children of the new node for elements with IDs
              elem.querySelectorAll('[id]').forEach((child) => {
                this.intersectionObserver.observe(child)
              })
            }
          })
        }
      }
    })

    // Start observing the render root for changes
    this.mutationObserver.observe(this.renderRoot, {
      childList: true,
      subtree: true,
    })

    // Observe all existing elements that have ids
    this.renderRoot.querySelectorAll('[id]').forEach((element) => {
      this.intersectionObserver.observe(element)
    })
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
      case 'scroll-sync':
        return this.handleScrollSyncMessage(data)
      default:
        throw new Error(`Unhandled received message type: ${type}`)
    }
  }

  /**
   * Handle a received `DomPatchMessage` message
   */
  private handleDomPatchMessage({ patch }: DomPatchMessage) {
    const { version, ops } = patch as unknown as FormatPatch

    // Check for non-sequential patch and request a reset patch if necessary
    const isReset = ops.length >= 1 && ops[0].type === 'reset'
    if (!isReset && version > this.version + 1) {
      this.requestReset()
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

    // Get the target element
    const documentRoot = this.renderRoot.querySelector('[root]')
    if (!documentRoot) {
      console.error('No document root found')
      return
    }

    // Update element
    if (isReset && this.failedMorph >= 2) {
      // If this is a reset patch and there was already two attempts
      // using morphing, then resort to replacing innerHTML of root
      documentRoot.innerHTML = new DOMParser().parseFromString(
        this.html,
        'text/html'
      ).body.firstElementChild.innerHTML
      this.failedMorph = 0
    } else {
      try {
        Idiomorph.morph(documentRoot, this.html)
        this.failedMorph = 0
      } catch (error) {
        // Any errors during morphing (i.e if somehow the HTML is invalid)
        // result in a reset request being sent to the server.
        console.log('While morphing DOM', error)
        this.failedMorph += 1
        this.requestReset()
      }
    }
  }

  /**
   * Send a request for a reset of the DOM HTML
   *
   * Used when an out of sequence patch is received, or when there is an error
   * then morphing HTML into the DOM (i.e. for some reason the currently maintained
   * state of the HTML is invalid)
   */
  private requestReset() {
    vscode.postMessage({ type: 'dom-reset' })
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
      ) as UIBaseClass & MarkerToggleInterface
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

  /**
   * Handle a received `ScrollSyncMessage` message
   */
  private handleScrollSyncMessage({ startId, cursorId }: ScrollSyncMessage) {
    // Prioritize cursor position if it exists
    if (cursorId) {
      const cursorElement = document.getElementById(cursorId)
      if (cursorElement) {
        // Check if element is already visible
        const rect = cursorElement.getBoundingClientRect()
        const isVisible = rect.top >= 0 && rect.bottom <= window.innerHeight

        if (!isVisible) {
          cursorElement.scrollIntoView({
            behavior: 'smooth',
            block: 'nearest',
          })
        }
      }
      return
    }

    // If no cursor, try to maintain viewport position using start/end elements
    if (startId) {
      const startElement = document.getElementById(startId)
      if (startElement) {
        const viewportHeight = window.innerHeight
        const rect = startElement.getBoundingClientRect()

        // Scroll to position the start element near the top with some padding
        window.scrollTo({
          top: rect.top + window.scrollY - viewportHeight * 0.1,
          behavior: 'smooth',
        })
      }
    }
  }
}
