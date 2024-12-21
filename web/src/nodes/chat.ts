import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property, query } from 'lit/decorators'

import { withTwind } from '../twind'

/**
 * Web component representing a Stencila `Chat` node
 *
 * A `Chat` node is similar to `Article` and `Prompt` nodes in that they
 * are usually (but not necessarily) the root node of a document.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/chat.md
 */
@customElement('stencila-chat')
@withTwind()
export class StencilaChat extends LitElement {
  @query('slot[name="content"]')
  contentSlot!: HTMLSlotElement

  contentObserver: MutationObserver

  override firstUpdated(): void {
    const slottedElement = this.contentSlot.assignedElements()[0]
    if (slottedElement) {
      this.contentObserver = new MutationObserver(() => {
        // TODO refine this for smoother transition
        // possibly try to add the typing effect for the text content?
        window.scrollTo({
          top: document.body.scrollHeight,
          behavior: 'smooth',
        })
      })
      this.contentObserver.observe(slottedElement, {
        subtree: true,
        childList: true,
      })
    }
  }

  override disconnectedCallback(): void {
    if (this.contentObserver) {
      this.contentObserver.disconnect()
    }
  }

  /**
   * Indicates that this is the root node of the document
   */
  @property({ type: Boolean })
  root: boolean

  @property()
  target?: string

  @property()
  prompt?: string

  override render() {
    const containerStyles = apply([
      'fixed bottom-0 left-0 z-10',
      'w-full',
      'bg-gray-100',
      'border-t border-black/20',
    ])

    return html`
      <div>
        <div class="fixed top-0 left-0 z-10 w-full">
          <slot name="model-parameters"></slot>
        </div>

        <div class="px-12 pb-[300px]">
          <slot name="content"></slot>
        </div>

        <div class=${containerStyles}>
          <div class="max-w-[400px] mx-auto">
            <stencila-chat-message-inputs
              message-id=${this.id}
            ></stencila-chat-message-inputs>
          </div>
        </div>
      </div>
    `
  }
}
