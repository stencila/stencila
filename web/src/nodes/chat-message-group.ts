import { html, css } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Entity } from './entity'

/**
 * Web component representing a Stencila `ChatMessageGroup`
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/chat-message-group.md
 */
@customElement('stencila-chat-message-group')
@withTwind()
export class ChatMessageGroup extends Entity {
  static override styles = css`
    ::slotted([slot='messages']) {
      display: flex;
      flex-direction: row;
      align-items: flex-start;
      justify-items: center;
      gap: 2rem;
    }
  `

  override render() {
    return html`<div class="flex justify-center overflow-x-auto mb-3">
      <slot name="messages"></slot>
    </div>`
  }
}
