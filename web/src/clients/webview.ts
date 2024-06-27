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

declare const vscode: VSCode

/**
 *
 */
export class WebViewClient {
  constructor(element: HTMLElement) {
    this.element = element
    this.setWindowListener()
  }

  private element: HTMLElement

  static postMessage(message: VSCodeMessage) {
    console.log('vscode api instance', vscode)
    vscode.postMessage(message)
  }

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
    console.log('scroll handler', this.element)
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
