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
    return html`
      <div class="fixed top-0 left-0 z-10 w-full">
        <slot name="model"></slot>
      </div>

      <div class="px-12 pb-[300px]">
        <slot name="content"></slot>
      </div>
    `
  }
}
