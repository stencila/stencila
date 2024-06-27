import { InlineTypeList } from '@stencila/types'

import { Entity } from '../nodes/entity'
import { ChipToggleInterface } from '../ui/nodes/mixins/toggle-chip'
import { UIBaseClass } from '../ui/nodes/mixins/ui-base-class'

type MessageType = 'scroll-to-element' | 'scroll-track'

type MessagePayload = {
  type: MessageType
  payload: {
    // the id of the node to scroll the view to
    scrollTarget?: string
  }
}

type VSCodeMessage = {
  command: string
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [k: string]: any
}

interface VSCode {
  postMessage(message: VSCodeMessage): void
}

// make sure the compiler is aware of the existing `vscode` api instance
declare const vscode: VSCode

/**
 * a client object to allow for sending recie
 */
export class WebViewClient {
  constructor(element: HTMLElement) {
    this.element = element
    this.setWindowListener()
  }

  private element: HTMLElement

  /**
   * Sends message to webview panel, via the vscode api instance
   */
  static postMessage(message: VSCodeMessage) {
    vscode.postMessage(message)
  }

  /**
   * add an event listener to the window instance for 'message'
   * events from the webview panel
   *
   * nb. any class methods used for/in the event callback must be bound to `this`
   * if they wish to use properties of `this`. using arrow function
   * syntax when declaring methods will achieve this.
   */
  private setWindowListener() {
    window.addEventListener('message', this.receiveMessage)
  }

  /**
   * Handles 'message' events sent from vscode to the `window`.
   * uses the event data to determine which handler fucntion to use.
   * @param e `Event` instance with `data` property carrying payload
   * @returns `void`
   */
  // !!!must be arrow function
  private receiveMessage = (e: Event & { data: MessagePayload }) => {
    const { type, payload } = e.data

    switch (type) {
      case 'scroll-to-element':
        if (payload.scrollTarget) {
          this.handleScroll(payload.scrollTarget)
        }
        break
      default:
        return
    }
  }

  /**
   * handles the 'scroll-to-element' message event
   * @param scrollTarget #id of target node
   */
  // !!!must be arrow function
  private handleScroll = (scrollTarget: string) => {
    const targetEl = this.element.querySelector(`#${scrollTarget}`) as Entity
    if (targetEl) {
      targetEl.scrollIntoView({
        block: 'start',
        inline: 'nearest',
        behavior: 'smooth',
      })

      let cardType = 'stencila-ui-block-on-demand'

      if (InlineTypeList.includes(scrollTarget.constructor.name)) {
        cardType = 'stencila-ui-inline-on-demand'
      }
      const card = targetEl.shadowRoot.querySelector(cardType) as UIBaseClass &
        ChipToggleInterface

      if (card) {
        card.openCard()
      }
    }
  }
}
