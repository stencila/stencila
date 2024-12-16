import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/chat/message-inputs'

type MessageStatus = 'empty' | 'edited' | 'sent' | 'recieved'

// MOVED TO CHAT MESSAGE FOR NOW
/**
 * Web component representing a Stencila Schema `InstructionMessage` node
 * Can be used both for new message input and displaying existing messages
 */
@customElement('stencila-instruction-message')
@withTwind()
export class InstructionMessage extends LitElement {
  @property({ type: String })
  model: string = 'model'

  @property({ type: Boolean })
  pending: boolean = false

  @property({ type: String, attribute: 'message-status' })
  messageStatus: MessageStatus = 'empty'

  @property({ type: Number, attribute: 'time-received' })
  timeReceived: number

  private renderMessage() {
    return html`<slot></slot>`
  }

  override render() {
    return this.messageStatus === 'empty'
      ? html`<stencila-message-input
          ?waiting=${this.pending}
        ></stencila-message-input>`
      : this.renderMessage()
  }
}
