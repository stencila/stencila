import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

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
      <slot name="model"></slot>
      <slot name="content"></slot>
    `
  }
}
